use three_d::*;

use battleground_construct::Construct;

mod construct_render;
use construct_render::ConstructRender;
use construct_render::RenderPass;

mod fence_material;
use fence_material::FenceMaterial;

// This solution is a bit heavy handed, but using the time from the frame input means we have no
// means of updating the time while we're in the frame calculation and thus can't break the
// construct simulation steps. So we use this external time provider that's either std::Time or the
// one provided through webassembly.
mod time_provider;

mod gui;

mod limiter;
use limiter::Limiter;

const PRINT_DURATIONS: bool = false;

use engine::EntityId;
#[derive(Debug)]
pub struct ViewerState {
    exiting: bool,
    paused: bool,
    previous_playback: f32,
    playback: f32,
    desired_speed: f32,
    selected: std::collections::HashSet<EntityId>,
    gui: gui::State,
}
impl Default for ViewerState {
    fn default() -> Self {
        Self {
            exiting: false,
            paused: false,
            previous_playback: 0.0,
            playback: 0.0,
            desired_speed: 1.0,
            selected: Default::default(),
            gui: Default::default(),
        }
    }
}

struct ConstructViewer {
    camera: Camera,
    context: three_d::core::Context,
    ambient_light: three_d::renderer::light::AmbientLight,
    directional_light: DirectionalLight,
    control: FlyControl,
    // control: OrbitControl,
    window: Window,

    construct: Construct,

    limiter: Limiter,

    construct_render: ConstructRender,
    printed_match_result: bool,
}

impl ConstructViewer {
    pub fn new(construct: Construct) -> Self {
        let window = Window::new(WindowSettings {
            title: "Battleground Construct".to_string(),
            min_size: (640, 480),
            // max_size: Some((1280, 720)),
            ..Default::default()
        })
        .unwrap();

        let context = window.gl();

        let limiter = Limiter::new();

        let camera = Camera::new_perspective(
            window.viewport(),
            vec3(-5.0, 2.0, 1.5), // position
            vec3(0.0, 0.0, -0.5), // target
            vec3(0.0, 0.0, 1.0),  // up
            degrees(45.0),
            0.1,
            1000.0,
        );

        /*
        let config = three_d::renderer::control::OrbitControlConfig {
            speed_orbit_horizontal: 0.1,
            speed_orbit_vertical: 0.1,
            speed_zoom: 2.0,
            speed_orbit_target_left: Some(0.1),
            speed_orbit_target_up: Some(0.1),
            ..Default::default()
        };

        let control = OrbitControl::new_with_config(config);
        */
        let control = FlyControl::new(0.1);

        let ambient_light =
            three_d::renderer::light::AmbientLight::new(&context, 0.2, Color::WHITE);
        let directional_light =
            DirectionalLight::new(&context, 1.5, Color::WHITE, &vec3(0.0, 0.2, -1.0));

        let construct_render: ConstructRender = ConstructRender::new();

        ConstructViewer {
            camera,
            context,
            ambient_light,
            directional_light,
            control,
            window,
            construct,
            limiter,
            construct_render,
            printed_match_result: false,
        }
    }

    // Consumes the viewer...
    fn view_loop(mut self) {
        let jump = 0.0;

        self.limiter.set_desired_speed(1.0);

        while self.construct.elapsed_as_f32() < jump {
            self.construct.update();
        }

        let mut gui = three_d::GUI::new(&self.context);

        let mut viewer_state = ViewerState::default();

        self.window.render_loop(move |mut frame_input: FrameInput| {
            let now = time_provider::Instant::now();
            // Run the limiter to update the construct.s
            if self.construct.can_update() {
                self.limiter.update(|| {
                    self.construct.update();
                    if self.construct.can_update() {
                        Some(self.construct.elapsed_as_f32())
                    } else {
                        None
                    }
                });
            }

            // This... should probably also not be here, but it's nice if the gui does something
            // more elegant with this at some point.
            if self.construct.is_match_finished() && !self.printed_match_result {
                let report = battleground_construct::config::wrap_up::create_wrap_up_report(
                    self.construct.world(),
                );
                println!("{report:#?}");
                self.printed_match_result = true;
            }

            if PRINT_DURATIONS {
                println!(
                    "construct update (not a single step!): {:1.8}, entities: {}",
                    now.elapsed().as_secs_f32(),
                    self.construct.world().entity_count()
                );
            }

            // Gui rendering.
            viewer_state.gui.update(&self.construct);
            gui.update(
                &mut frame_input.events,
                frame_input.accumulated_time,
                frame_input.viewport,
                frame_input.device_pixel_ratio,
                |ctx| {
                    gui::window_match(ctx, &self.construct, &mut viewer_state.gui);
                    gui::window_play(ctx, &self.construct, &mut viewer_state, &mut self.limiter);
                    gui::top_bar(ctx, &mut viewer_state);
                },
            );

            self.camera.set_viewport(frame_input.viewport);
            self.control
                .handle_events(&mut self.camera, &mut frame_input.events);

            if viewer_state.previous_playback != viewer_state.playback {
                println!("Something changed, new value is {}", viewer_state.playback);
                viewer_state.previous_playback = viewer_state.playback;
                self.construct.recording_seek(viewer_state.playback);
            }
            viewer_state.previous_playback = self.construct.elapsed_as_f32();
            viewer_state.playback = viewer_state.previous_playback;
            self.limiter.set_desired_speed(viewer_state.desired_speed);

            for e in frame_input.events.iter() {
                match *e {
                    three_d::Event::KeyPress {
                        kind: Key::Space,
                        handled: false,
                        ..
                    } => {
                        viewer_state.paused = !viewer_state.paused;
                        self.limiter.set_paused(viewer_state.paused);
                    }
                    three_d::Event::KeyPress {
                        kind: Key::Q,
                        handled: false,
                        modifiers: Modifiers { ctrl: true, .. },
                        ..
                    } => {
                        viewer_state.exiting = true;
                    }
                    three_d::Event::MousePress {
                        button,
                        position,
                        modifiers,
                        ..
                    } => {
                        if button == three_d::renderer::control::MouseButton::Middle {
                            let position = three_d::control::control_position_to_viewport_position(
                                position,
                                frame_input.device_pixel_ratio,
                                &frame_input.viewport,
                            );
                            let pos = self.camera.position_at_pixel(position);
                            let dir = self.camera.view_direction_at_pixel(position);

                            // Now that we have the ray, we can calculate what and if it hit something.
                            // We need the construct to do that though.
                            let intersects = self.construct.select_intersect(&pos, &dir);

                            // Shooting one ray through multiple entities is hard... allow shift to
                            // add or remove from the selected set.
                            if modifiers.shift {
                                // toggle whatever is clicked.
                                for v in intersects {
                                    if viewer_state.selected.contains(&v) {
                                        viewer_state.selected.remove(&v);
                                    } else {
                                        viewer_state.selected.insert(v);
                                    }
                                }
                            } else {
                                // just assign the new selection.
                                viewer_state.selected.clear();
                                for v in intersects {
                                    viewer_state.selected.insert(v);
                                }
                            }
                            println!("viewer_state.selected: {:?}", viewer_state.selected);
                        }
                    }
                    _ => {}
                }
            }

            let screen = frame_input.screen();

            let now = time_provider::Instant::now();

            if let Some((pos, target)) = self
                .construct_render
                .camera_view(&self.camera, &self.construct)
            {
                self.camera.set_view(pos, target, vec3(0.0, 0.0, 1.0));
            }
            self.construct_render.render(
                &self.camera,
                &self.context,
                &self.construct,
                &viewer_state.selected,
            );

            if PRINT_DURATIONS {
                println!("elements: {}", now.elapsed().as_secs_f64());
            }

            let now = time_provider::Instant::now();

            /* The rendering steps will look something like this:
                0) Prerender shadow maps
                A1) Scene render (targets framebuffer)
                B1) Render depth of non-emissives into depth texture
                B2) Render emissives into color texture (use B1 as depth texture)
                C) Render fence meshes to framebuffer (with bound depth texture)
                D) Write B2 into A additively
            */

            // 0) Prerender shadow maps
            // Skip the ground plane in the shadow map, otherwise we get no resolution.
            self.directional_light.generate_shadow_map(
                2048,
                &self.construct_render.geometries(RenderPass::ShadowMaps),
            );

            // A) Render normal scene
            screen
                .clear(ClearState::color_and_depth(0.8, 0.8, 0.8, 1.0, 1.0))
                .render(
                    &self.camera,
                    &self.construct_render.objects(RenderPass::BaseScene),
                    &[&self.ambient_light, &self.directional_light],
                );

            // B1) Render depth buffer with non-emissives
            let mut depth_texture = DepthTexture2D::new::<f32>(
                &self.context,
                frame_input.viewport.width,
                frame_input.viewport.height,
                Wrapping::ClampToEdge,
                Wrapping::ClampToEdge,
            );

            let write_depth_material = ColorMaterial {
                render_states: RenderStates {
                    write_mask: WriteMask::DEPTH,
                    ..Default::default()
                },
                ..Default::default()
            };
            depth_texture
                .as_depth_target()
                .clear(ClearState::default())
                .render_with_material(
                    &write_depth_material,
                    &self.camera,
                    &self.construct_render.objects(RenderPass::NonGlowDepths),
                    &[],
                );

            // B2) Render emissives to color texture
            let mut emissive_texture = Texture2D::new_empty::<[u8; 4]>(
                &self.context,
                frame_input.viewport.width,
                frame_input.viewport.height,
                Interpolation::Nearest,
                Interpolation::Nearest,
                None,
                Wrapping::ClampToEdge,
                Wrapping::ClampToEdge,
            );

            RenderTarget::new(
                emissive_texture.as_color_target(None),
                depth_texture.as_depth_target(),
            )
            .render(
                &self.camera,
                &self.construct_render.objects(RenderPass::GlowSources),
                &[],
            );

            // C) Render fence meshes to framebuffer (with bound depth texture)
            let fence_material = FenceMaterial::new(&depth_texture);
            screen.render_with_material(
                &fence_material,
                &self.camera,
                &self.construct_render.objects(RenderPass::Fences),
                &[],
            );

            // D) Write B2 into A additively
            screen
                .write(|| {
                    apply_effect(
                        &self.context,
                        include_str!("shaders/bloom_effect.frag"),
                        RenderStates {
                            write_mask: WriteMask::COLOR,
                            blend: Blend::ADD,
                            cull: Cull::Back,
                            depth_test: DepthTest::Always,
                        },
                        self.camera.viewport(),
                        |program| {
                            program.use_texture("emissive_buffer", &emissive_texture);
                        },
                    )
                })
                .write(|| gui.render());

            //----------------------------------------------------------------

            // self.construct_render.reset_instances();

            if PRINT_DURATIONS {
                println!("render: {}", now.elapsed().as_secs_f64());
            }

            if viewer_state.exiting {
                // This does not just exit the render loop, it also shuts down the program.
                return FrameOutput {
                    exit: true,
                    ..Default::default()
                };
            }

            FrameOutput::default()
        });
    }
}

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = std::env::args().collect::<Vec<String>>();

    // Preserve the trailer...
    let tree_trailer = args.len() >= 2 && args.get(1).unwrap() == "--tree-trailer";

    let construct = if tree_trailer {
        let mut construct = Construct::new();
        battleground_construct::config::tree_trailer::populate_tree_trailer(&mut construct);
        construct
    } else {
        let config = battleground_construct::config::cli::parse_setup_args()?;
        battleground_construct::config::setup::setup(config)?
    };

    let viewer = ConstructViewer::new(construct);

    // view loop consumes the viewer... :|
    viewer.view_loop();
    Ok(())
}

// Entry point for wasm
#[cfg(target_arch = "wasm32")]
mod wasm32 {
    use js_sys::{ArrayBuffer, Uint8Array};
    use log::info;
    use wasm_bindgen::prelude::*;
    use wasm_bindgen::JsCast;
    use wasm_bindgen_futures::JsFuture;
    use web_sys::{Blob, UrlSearchParams};

    // https://github.com/rustwasm/wasm-bindgen/issues/1292

    async fn get_data() -> Result<Vec<u8>, JsValue> {
        use web_sys::{Request, RequestInit, RequestMode, Response};

        fn get_window() -> Result<web_sys::Window, JsValue> {
            web_sys::window().ok_or_else(|| JsValue::from_str("couldn't get window"))
        }
        // Fetch the recording.
        let mut opts = RequestInit::new();
        opts.method("GET");
        opts.mode(RequestMode::Cors);

        let location_origin = get_window()?.location().search()?;
        let url_params = UrlSearchParams::new_with_str(&location_origin)?;

        // info!("{}", location_origin);
        // let url = format!("/pkg/recording.bin");
        let url = url_params
            .get("url")
            .ok_or_else(|| JsValue::from_str("could not find url parameter"))?;

        let request = Request::new_with_str_and_init(&url, &opts)?;

        request
            .headers()
            .set("Accept", "application/octet-stream")?;

        let window = web_sys::window().unwrap();
        let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;

        // `resp_value` is a `Response` object.
        assert!(resp_value.is_instance_of::<Response>());
        let resp: Response = resp_value.dyn_into().unwrap();

        // Convert this other `Promise` into a rust `Future`.
        let blob = JsFuture::from(resp.blob()?).await?;
        // Ok(vec![])
        let blob = Blob::from(blob);
        info!("Blob size {}", blob.size()); // looks good!

        // read_as_array_buffer(&self, blob: &Blob) -> Result<(), JsValue>
        // let file_reader = FileReader::new()?;
        // file_reader.read_as_array_buffer(&blob)?;

        let arr = JsFuture::from(blob.array_buffer()).await?;
        // info!("{:?}", arr);
        let array: ArrayBuffer = arr.into();
        // info!("{:?}", array);

        // let array = ArrayBuffer::from(file_reader.result()?);
        let array = Uint8Array::new(&array);
        // info!("array as uin8tarray {:?}", array);

        // let mut as_vec = Vec::with_capacity(array.byte_length() as usize);
        // as_vec.resize(array.byte_length() as usize, 0);
        let mut as_vec = vec![0; array.byte_length() as usize];
        array.copy_to(&mut as_vec[..]);

        Ok(as_vec)
    }

    #[wasm_bindgen(start)]
    pub async fn start() -> Result<(), JsValue> {
        console_log::init_with_level(log::Level::Debug).unwrap();

        use log::info;
        info!("Logging works!");

        std::panic::set_hook(Box::new(console_error_panic_hook::hook));

        let data = get_data().await;

        let construct = if let Ok(data) = data {
            info!("Found data!");
            battleground_construct::config::setup::setup_playback_slice(&data).unwrap()
        } else {
            info!("No data, setting up the playground!");
            let mut construct = battleground_construct::Construct::new();
            battleground_construct::config::default::add_components(&mut construct.world);
            battleground_construct::config::default::add_systems(&mut construct.systems);
            battleground_construct::config::playground::populate_dev_world(&mut construct);
            construct
        };

        let viewer = super::ConstructViewer::new(construct);

        // view loop consumes the viewer... :|
        viewer.view_loop();

        Ok(())
    }
}
