use three_d::*;

use battleground_construct::Construct;
use battleground_construct::components;

struct Limiter {
    pub period: std::time::Duration,
    pub last_time: std::time::Instant
}
impl Limiter {
    pub fn new(period: f32) -> Self {
        Limiter{
            period: std::time::Duration::from_secs_f32(period),
            last_time: std::time::Instant::now()
        }
    }
    pub fn elapsed(&mut self) -> bool {
        let now = std::time::Instant::now();
        if (now - self.last_time) > self.period {
            self.last_time  = now;
            return true;
        }
        return false;
    }
}

pub fn main() {
    let window = Window::new(WindowSettings {
        title: "Shapes!".to_string(),
        max_size: Some((1280, 720)),
        ..Default::default()
    })
    .unwrap();
    let context = window.gl();

    let mut construct = Construct::new();

    let mut camera = Camera::new_perspective(
        window.viewport(),
        vec3(5.0, 2.0, 2.5),
        vec3(0.0, 0.0, -0.5),
        vec3(0.0, 1.0, 0.0),
        degrees(45.0),
        0.1,
        1000.0,
    );
    let mut control = OrbitControl::new(*camera.target(), 1.0, 100.0);

    let mut sphere = Gm::new(
        Mesh::new(&context, &CpuMesh::sphere(16)),
        PhysicalMaterial::new_transparent(
            &context,
            &CpuMaterial {
                albedo: Color {
                    r: 255,
                    g: 0,
                    b: 0,
                    a: 200,
                },
                ..Default::default()
            },
        ),
    );
    sphere.set_transformation(Mat4::from_translation(vec3(0.0, 0.0, 0.0)) * Mat4::from_scale(0.2));


    let mut cpu_plane = CpuMesh::square();
    let mut ground_plane = Gm::new(
        Mesh::new(&context, &cpu_plane),
        PhysicalMaterial::new_opaque(
            &context,
            &CpuMaterial {
                albedo: Color::new_opaque(128, 128, 128),
                ..Default::default()
            },
        ),
    );
    ground_plane.set_transformation(Mat4::from_translation(vec3(0.0, 0.0, 0.0)) * Mat4::from_scale(1000.0) * Mat4::from_angle_x(Rad(std::f32::consts::PI / 2.0)));

    let mut deferred_plane = Gm::new(
        Mesh::new(&context, &cpu_plane),
        DeferredPhysicalMaterial::from_physical_material(&ground_plane.material),
    );
    deferred_plane.set_transformation(Mat4::from_translation(vec3(0.0, 0.0, 0.0)) * Mat4::from_scale(1000.0) * Mat4::from_angle_x(Rad(std::f32::consts::PI / 2.0)));


    let mut cube = Gm::new(
        Mesh::new(&context, &CpuMesh::cube()),
        three_d::renderer::material::PhysicalMaterial::new_opaque(
            &context,
            &CpuMaterial {
                albedo: Color {
                    r: 0,
                    g: 0,
                    b: 255,
                    a: 255,
                },
                ..Default::default()
            },
        ),
    );
    cube.set_transformation(Mat4::from_translation(vec3(0.0, 0.0, 0.0)) * Mat4::from_scale(0.2));



    let light0 = three_d::renderer::light::AmbientLight::new(&context, 0.3, Color::WHITE);
    let mut light1 = DirectionalLight::new(&context, 1.0, Color::WHITE, &vec3(0.0, -5.5, 0.5));


    let mut limiter = Limiter::new(0.01);

    window.render_loop(move |mut frame_input: FrameInput| {
        if limiter.elapsed() {
            construct.update();
        }

        // Set the cube's position... 
        let vehicle_id = construct.vehicle_id();
        let mut pose = *construct.world().component::<components::pose::Pose>(&vehicle_id).expect("Should have a pose for the vehicle");
        pose.h.w[1] += 2.0;
        cube.set_transformation(pose.h);

        camera.set_viewport(frame_input.viewport);
        control.handle_events(&mut camera, &mut frame_input.events);

        light1.generate_shadow_map(1024, &cube);

        frame_input
            .screen()
            .clear(ClearState::color_and_depth(0.8, 0.8, 0.8, 1.0, 1.0))
            .render(
                &camera,
                deferred_plane
                    .into_iter()
                    .chain(&cube)
                    .chain(&sphere),
                &[&light0, &light1],
            );

        FrameOutput::default()
    });
}
