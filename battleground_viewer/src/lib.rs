use three_d::*;

use battleground_construct::components;
use battleground_construct::display;
use battleground_construct::display::primitives::Drawable;
use battleground_construct::Construct;
use engine::prelude::*;

struct Limiter {
    pub period: std::time::Duration,
    pub last_time: std::time::Instant,
}

impl Limiter {
    pub fn new(period: f32) -> Self {
        Limiter {
            period: std::time::Duration::from_secs_f32(period),
            last_time: std::time::Instant::now(),
        }
    }
    pub fn elapsed(&mut self) -> bool {
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
    control: OrbitControl,
    window: Window,

    construct: Construct,

    limiter: Limiter,
}

fn element_to_gm(
    context: &Context,
    el: &display::primitives::Element,
) -> Gm<Mesh, PhysicalMaterial> {
    let mut mesh = {
        match el.primitive {
            display::primitives::Primitive::Cuboid(cuboid) => {
                let mut m = CpuMesh::cube();
                // Returns an axis aligned unconnected cube mesh with positions in the range [-1..1] in all axes.
                // So default box is not identity.
                m.transform(&Mat4::from_nonuniform_scale(
                    cuboid.length / 2.0,
                    cuboid.width / 2.0,
                    cuboid.height / 2.0,
                ))
                .unwrap();
                m
            }
            display::primitives::Primitive::Sphere(sphere) => {
                let mut m = CpuMesh::sphere(16);
                m.transform(&Mat4::from_scale(sphere.radius)).unwrap();
                m
            }
            display::primitives::Primitive::Cylinder(cylinder) => {
                let mut m = CpuMesh::cylinder(16);
                m.transform(&Mat4::from_nonuniform_scale(
                    cylinder.height,
                    cylinder.radius,
                    cylinder.radius,
                ))
                .unwrap();
                m
            }
        }
    };
    mesh.transform(&el.transform).unwrap();
    let mesh = Mesh::new(&context, &mesh);
    let mut drawable = Gm::new(
        mesh,
        three_d::renderer::material::PhysicalMaterial::new_opaque(
            &context,
            &CpuMaterial {
                albedo: Color {
                    r: el.color.r,
                    g: el.color.g,
                    b: el.color.b,
                    a: el.color.a,
                },
                emissive: Color {
                    r: el.color.r,
                    g: el.color.g,
                    b: el.color.b,
                    a: el.color.a,
                },
                //roughness: 0.0,
                //metallic: 0.0,
                ..Default::default()
            },
        ),
    );
    drawable
}

fn component_to_meshes<C: Component + Drawable + 'static>(
    context: &Context,
    construct: &Construct,
) -> Vec<Gm<Mesh, PhysicalMaterial>> {
    let mut res: Vec<Gm<Mesh, PhysicalMaterial>> = vec![];

    for (element_id, component_with_drawables) in construct.world().component_iter::<C>() {
        let mut meshes = component_with_drawables
            .drawables()
            .iter()
            .map(|v| element_to_gm(context, v))
            .collect::<Vec<Gm<Mesh, PhysicalMaterial>>>();

        // Get the world pose for this entity, and thus mesh.
        let world_pose = construct.entity_pose(&element_id);

        // Collapse this up the stack.
        for gm in meshes.iter_mut() {
            let current = gm.geometry.transformation();
            let updated = *(world_pose) * current;
            gm.geometry.set_transformation(updated);
        }
        res.append(&mut meshes);
    }
    res
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

        let mut camera = Camera::new_perspective(
            window.viewport(),
            vec3(5.0, 2.0, 2.5),
            vec3(0.0, 0.0, -0.5),
            vec3(0.0, 0.0, 1.0),
            degrees(45.0),
            0.1,
            1000.0,
        );
        let mut control = OrbitControl::new(*camera.target(), 1.0, 100.0);

        let ambient_light =
            three_d::renderer::light::AmbientLight::new(&context, 0.1, Color::WHITE);
        let mut directional_light =
            DirectionalLight::new(&context, 1.5, Color::WHITE, &vec3(0.0, 0.0, -1.0));

        ConstructViewer {
            camera,
            context,
            ambient_light,
            directional_light,
            control,
            window,
            construct,
            limiter,
        }
    }

    // Consumes the viewer...
    fn view_loop(mut self) -> () {
        self.window.render_loop(move |mut frame_input: FrameInput| {
            if self.limiter.elapsed() {
                self.construct.update();
                let (_entity, mut clock) = self
                    .construct
                    .world()
                    .component_iter_mut::<components::clock::Clock>()
                    .next()
                    .expect("Should have one clock");
                // println!("Realtime ratio: {}", clock.ratio_of_realtime());
            }

            self.camera.set_viewport(frame_input.viewport);
            self.control
                .handle_events(&mut self.camera, &mut frame_input.events);

            let mut screen = frame_input.screen();
            screen.clear(ClearState::color_and_depth(0.8, 0.8, 0.8, 1.0, 1.0));

            let elements = Self::render_construct(&self.context, &self.construct);

            // Skip the ground plane in the shadow map, otherwise we get no resolution.
            self.directional_light
                .generate_shadow_map(2048, elements.iter().skip(1).map(|x| &x.geometry));

            screen.render(
                &self.camera,
                elements.iter(),
                &[&self.ambient_light, &self.directional_light],
            );

            FrameOutput::default()
        });
    }

    pub fn render_construct(
        context: &Context,
        construct: &Construct,
    ) -> Vec<Gm<Mesh, PhysicalMaterial>> {
        let mut res = vec![];

        let mut ground_plane = Gm::new(
            Mesh::new(&context, &CpuMesh::square()),
            PhysicalMaterial::new_opaque(
                &context,
                &CpuMaterial {
                    albedo: Color::new_opaque(128, 128, 128),
                    ..Default::default()
                },
            ),
        );
        ground_plane.set_transformation(
            Mat4::from_translation(vec3(0.0, 0.0, 0.0)) * Mat4::from_scale(1000.0),
        );
        res.push(ground_plane);

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
        cube.set_transformation(
            Mat4::from_translation(vec3(0.0, 0.0, 1.0)) * Mat4::from_scale(0.2),
        );
        res.push(cube);

        res.append(&mut component_to_meshes::<display::tank_body::TankBody>(
            context, construct,
        ));
        res.append(
            &mut component_to_meshes::<display::tank_turret::TankTurret>(context, construct),
        );

        res.append(
            &mut component_to_meshes::<display::tank_barrel::TankBarrel>(context, construct),
        );


        res
    }
}

pub fn main() {
    let construct = Construct::new();
    let viewer = ConstructViewer::new(construct);

    // view loop consumes the viewer... :|
    viewer.view_loop();
}
