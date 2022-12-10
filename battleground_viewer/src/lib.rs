use three_d::*;

use battleground_construct;
use battleground_construct::Construct;

mod construct_render;
use construct_render::ConstructRender;

const PRINT_DURATIONS: bool = false;

pub struct Limiter {
    pub period: std::time::Duration,
    pub last_time: std::time::Instant,
    pub epoch: std::time::Instant,
}

impl Limiter {
    pub fn new(period: f32) -> Self {
        Limiter {
            epoch: std::time::Instant::now(),
            period: std::time::Duration::from_secs_f32(period),
            last_time: std::time::Instant::now(),
        }
    }

    pub fn elapsed_as_f32(&self) -> f32 {
        self.epoch.elapsed().as_secs_f32()
    }

    pub fn rate_elapsed(&mut self) -> bool {
        let now = std::time::Instant::now();
        if (now - self.last_time) > self.period {
            self.last_time = now;
            return true;
        }
        return false;
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

        let limiter = Limiter::new(0.001);

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
        let mut control = FlyControl::new(0.1);

        let ambient_light =
            three_d::renderer::light::AmbientLight::new(&context, 0.1, Color::WHITE);
        let directional_light =
            DirectionalLight::new(&context, 1.5, Color::WHITE, &vec3(0.0, 0.2, -1.0));

        let construct_render: ConstructRender = ConstructRender::new(&context);

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
        }
    }

    // Consumes the viewer...
    fn view_loop(mut self) -> () {
        let jump = 0.0;
        // let stop_sim_at = 5.023; // second impact.
        let stop_sim_at = 2000.3;
        let timespeed = 0.1;
        while self.construct.elapsed_as_f32() < jump {
            self.construct.update();
        }
        // battleground_construct::systems::velocity_pose::print_poses.store(true, std::sync::atomic::Ordering::Relaxed);

        self.window.render_loop(move |mut frame_input: FrameInput| {
            while self.construct.elapsed_as_f32() < stop_sim_at
                && (self.construct.elapsed_as_f32() * (1.0 / timespeed))
                    < (self.limiter.elapsed_as_f32() + jump)
            {
                let now = std::time::Instant::now();
                self.construct.update();
                if PRINT_DURATIONS {
                    println!(
                        "construct taken: {:1.8}, entities: {}",
                        now.elapsed().as_secs_f32(),
                        self.construct.world().entity_count()
                    );
                }
            }

            /*
            if self.limiter.rate_elapsed() {
                let (_entity, clock) = self
                    .construct
                    .world()
                    .component_iter_mut::<battleground_construct::components::clock::Clock>()
                    .next()
                    .expect("Should have one clock");
                println!("Realtime ratio: {}", clock.ratio_of_realtime());
            }
            */

            self.camera.set_viewport(frame_input.viewport);
            self.control
                .handle_events(&mut self.camera, &mut frame_input.events);

            let screen = frame_input.screen();
            screen.clear(ClearState::color_and_depth(0.8, 0.8, 0.8, 1.0, 1.0));

            let now = std::time::Instant::now();
            self.construct_render
                .render(&self.camera, &self.context, &self.construct);

            if PRINT_DURATIONS {
                println!("elements: {}", now.elapsed().as_secs_f64());
            }

            // Skip the ground plane in the shadow map, otherwise we get no resolution.
            self.directional_light
                .generate_shadow_map(2048, self.construct_render.shadow_meshes());

            let now = std::time::Instant::now();

            screen.render(
                &self.camera,
                &self.construct_render.objects(),
                &[&self.ambient_light, &self.directional_light],
            );

            //----------------------------------------------------------------

            // self.construct_render.reset_instances();

            if PRINT_DURATIONS {
                println!("render: {}", now.elapsed().as_secs_f64());
            }

            FrameOutput::default()
        });
    }
}

pub fn main() {
    let construct = Construct::new();
    let viewer = ConstructViewer::new(construct);

    // view loop consumes the viewer... :|
    viewer.view_loop();
}
