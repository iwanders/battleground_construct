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

const PRINT_DURATIONS: bool = false;

pub struct Limiter {
    desired_speed: f32,
    real_speed: f32,
    is_paused: bool,
    last_update_time: time_provider::Instant,
    last_construct_time: f32,
    update_deadline: time_provider::Duration,
}

impl Default for Limiter {
    fn default() -> Self {
        Self::new()
    }
}

impl Limiter {
    pub fn new() -> Self {
        Limiter {
            desired_speed: 1.0,
            real_speed: 1.0,
            last_construct_time: 0.0,
            is_paused: false,
            last_update_time: time_provider::Instant::now(),
            update_deadline: time_provider::Duration::from_secs_f64(1.0 / 60.0),
        }
    }

    pub fn set_paused(&mut self, paused: bool) {
        self.is_paused = paused;
    }

    pub fn is_paused(&self) -> bool {
        self.is_paused
    }

    pub fn set_desired_speed(&mut self, speed: f32) {
        self.desired_speed = speed;
    }

    /// Real speed is always a bit off from the desired speed, that is expected as the construct
    /// uses constant steps.
    pub fn real_speed(&self) -> f32 {
        self.real_speed
    }

    pub fn update<F: FnMut() -> Option<f32>>(&mut self, mut v: F) {
        if self.is_paused {
            self.last_update_time = time_provider::Instant::now();
            self.real_speed = 0.0;
            return;
        }

        let start_of_update = time_provider::Instant::now();

        let time_since_last = (start_of_update - self.last_update_time).as_secs_f32();
        let desired_construct_change = self.desired_speed * time_since_last;
        let desired_construct_finish_time = self.last_construct_time + desired_construct_change;
        let start_construct_time = self.last_construct_time;

        if desired_construct_finish_time > start_construct_time {
            loop {
                if start_of_update.elapsed() >= self.update_deadline {
                    // We didn't meet the update deadline, well... bummer.
                    // println!("Didn't meet rate");
                    break;
                }
                if let Some(new_time) = v() {
                    self.last_construct_time = new_time;
                    if self.last_construct_time >= desired_construct_finish_time {
                        break;
                    }
                } else {
                    break;
                }
            }
        }
        // Calculate the real speed we achieved.
        self.real_speed = (self.last_construct_time - start_construct_time) / time_since_last;
        self.last_update_time = time_provider::Instant::now();
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
            max_size: Some((1280, 720)),
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
            three_d::renderer::light::AmbientLight::new(&context, 0.1, Color::WHITE);
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

        use engine::EntityId;
        #[derive(Default, Debug)]
        struct ViewerState {
            exiting: bool,
            paused: bool,
            selected: std::collections::HashSet<EntityId>,
        }
        let mut viewer_state = ViewerState::default();

        #[cfg(target_arch = "wasm32")]
        let mut wasm_state = wasm32::State::default();

        self.window.render_loop(move |mut frame_input: FrameInput| {
            #[cfg(target_arch = "wasm32")]
            wasm32::view_loop(&mut self.construct, &mut wasm_state);

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
            gui.update(
                &mut frame_input.events,
                frame_input.accumulated_time,
                frame_input.viewport,
                frame_input.device_pixel_ratio,
                |gui_context| {
                    use three_d::egui::*;
                    egui::TopBottomPanel::top("my_panel").show(gui_context, |ui| {
                        menu::bar(ui, |ui| {
                            ui.menu_button("Construct", |ui| {
                                if ui.button("Quit").clicked() {
                                    viewer_state.exiting = true;
                                }
                            });
                            ui.with_layout(
                                egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
                                |ui| {
                                    ui.label(if viewer_state.selected.is_empty() {
                                        "select with middle click".to_owned()
                                    } else {
                                        format!("{:?}", viewer_state.selected)
                                    });
                                },
                            );
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    ui.menu_button(
                                        format!(
                                            "{:.2} x {:.2}",
                                            self.construct.elapsed_as_f32(),
                                            self.limiter.real_speed()
                                        ),
                                        |ui| {
                                            let label = if viewer_state.paused {
                                                "Resume"
                                            } else {
                                                "Pause"
                                            };
                                            if ui.button(label).clicked() {
                                                viewer_state.paused = !viewer_state.paused;
                                                self.limiter.set_paused(viewer_state.paused);
                                                ui.close_menu();
                                            }
                                            if ui.button("x0.1").clicked() {
                                                self.limiter.set_desired_speed(0.1);
                                                ui.close_menu();
                                            }
                                            if ui.button("x0.25").clicked() {
                                                self.limiter.set_desired_speed(0.25);
                                                ui.close_menu();
                                            }
                                            if ui.button("x1.0").clicked() {
                                                self.limiter.set_desired_speed(1.0);
                                                ui.close_menu();
                                            }
                                            if ui.button("x2.0").clicked() {
                                                self.limiter.set_desired_speed(2.0);
                                                ui.close_menu();
                                            }
                                        },
                                    );
                                },
                            );
                        });
                    });
                },
            );

            self.camera.set_viewport(frame_input.viewport);
            self.control
                .handle_events(&mut self.camera, &mut frame_input.events);

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
    use js_sys::{ArrayBuffer, Uint8Array, Promise};
    use log::info;
    use wasm_bindgen::prelude::*;
    use wasm_bindgen::JsCast;
    use wasm_bindgen_futures::JsFuture;
    use web_sys::{Blob, UrlSearchParams, Event, Response};

    // https://github.com/rustwasm/wasm-bindgen/issues/1292

    #[derive(Default)]
    pub struct State {
        started_request: bool,
        recording: Option<Vec<u8>>,
        req: Option<Response>,
        success_closure: Option<Box<Closure<dyn FnMut(JsValue)>>>
    }

    pub fn view_loop(construct: &mut super::Construct, state: &mut State) {
        info!("loop!");
        use futures::future::FutureExt;
        if !state.started_request {

            fn foo() -> Result<Promise, JsValue> {
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
                Ok(window.fetch_with_request(&request))
            }
            let v = foo();
            match v {
                Ok(p) => {
                    state.success_closure = Some(Box::new(Closure::new(
                        |resp_value: JsValue| {
                            info!("Jsvalue on then from promise {:?}", resp_value);
                            assert!(resp_value.is_instance_of::<Response>());
                            let resp: Response = resp_value.dyn_into().unwrap();
                            info!("resp {:?}", resp);
                        }
                    )));
                    // This returns a promise we don't await...
                    p.then(&*state.success_closure.as_ref().unwrap());
                },
                _ => panic!(),
            }
            
            
        }


        // let bg = battleground_construct::config::setup::setup_playback_slice(&data).unwrap();
    }

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

        // let construct = if let Ok(data) = data {
        // info!("Found data!");
        // battleground_construct::config::setup::setup_playback_slice(&data).unwrap()
        // } else {
        info!("No data, setting up the playground!");
        let mut construct = battleground_construct::Construct::new();
        battleground_construct::config::default::add_components(&mut construct.world);
        battleground_construct::config::default::add_systems(&mut construct.systems);
        battleground_construct::config::playground::populate_dev_world(&mut construct);
        // construct
        // };

        let viewer = super::ConstructViewer::new(construct);

        // view loop consumes the viewer... :|
        viewer.view_loop();

        Ok(())
    }
}
