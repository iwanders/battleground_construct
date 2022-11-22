use three_d::*;

use battleground_construct::display;
use battleground_construct::display::primitives::Drawable;
use battleground_construct::util::cgmath::ToQuaternion;
use battleground_construct::Construct;
use engine::prelude::*;

const PRINT_DURATIONS: bool = false;

struct Limiter {
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

    pub fn elapsed_as_f64(&self) -> f64 {
        self.epoch.elapsed().as_secs_f64()
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
    // control: FlyControl,
    control: OrbitControl,
    window: Window,

    construct: Construct,

    limiter: Limiter,

    persistence: RenderPersistence,
}

struct InstancedEntity {
    gm: three_d::renderer::object::Gm<three_d::renderer::geometry::InstancedMesh, PhysicalMaterial>,
    transforms: Vec<Mat4>,
    colors: Vec<Color>,
}
impl InstancedEntity {
    pub fn new(context: &Context, cpu_mesh: &CpuMesh) -> Self {
        let instances: three_d::renderer::geometry::Instances = Default::default();
        InstancedEntity {
            gm: Gm::new(
                InstancedMesh::new(context, &instances, cpu_mesh),
                three_d::renderer::material::PhysicalMaterial::new(
                    &context,
                    &CpuMaterial {
                        albedo: Color {
                            r: 255,
                            g: 255,
                            b: 255,
                            a: 255,
                        },
                        ..Default::default()
                    },
                ),
            ),
            transforms: vec![],
            colors: vec![],
        }
    }

    pub fn gm(
        &self,
    ) -> &three_d::renderer::object::Gm<three_d::renderer::geometry::InstancedMesh, PhysicalMaterial>
    {
        &self.gm
    }

    pub fn update_instances(&mut self) {
        let mut instances: three_d::renderer::geometry::Instances = Default::default();
        instances.translations = self
            .transforms
            .iter()
            .map(|m| m.w.truncate())
            .collect::<_>();

        // The transforms we have a homogeneous matrices, so the top left 3x3 is a rotation matrix.
        // We need to express that as a quaternion here.
        instances.rotations = Some(
            self.transforms
                .iter()
                .map(|m| m.to_quaternion())
                .collect::<_>(),
        );

        // Scaling is not done, this is ALWAYS done in the mesh itself, since all transforms are
        // homogeneous transforms.
        instances.colors = Some(self.colors.clone());
        self.gm.geometry.set_instances(&instances);
    }
}

#[derive(Default)]
struct RenderPersistence {
    static_gms: Vec<Gm<Mesh, PhysicalMaterial>>,
    instanced_meshes:
        std::collections::HashMap<crate::display::primitives::Primitive, InstancedEntity>,
}

impl RenderPersistence {
    pub fn update_instances(&mut self) {
        for instance_entity in self.instanced_meshes.values_mut() {
            instance_entity.update_instances()
        }
    }

    pub fn reset_instances(&mut self) {
        self.instanced_meshes.clear();
    }

    pub fn populate_default(&mut self, context: &Context) {
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
        self.static_gms.push(ground_plane);

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
        self.static_gms.push(cube);

        let mut cube = Gm::new(
            Mesh::new(&context, &CpuMesh::cube()),
            three_d::renderer::material::PhysicalMaterial::new_opaque(
                &context,
                &CpuMaterial {
                    albedo: Color {
                        r: 255,
                        g: 0,
                        b: 0,
                        a: 255,
                    },
                    ..Default::default()
                },
            ),
        );
        cube.set_transformation(
            Mat4::from_translation(vec3(1.0, 0.0, 0.0)) * Mat4::from_scale(0.2),
        );
        self.static_gms.push(cube);
    }
}

fn elements_to_render(
    persistence: &mut RenderPersistence,
    context: &Context,
    el: &display::primitives::Element,
    entity_transform: &Matrix4<f32>,
) {
    if !persistence.instanced_meshes.contains_key(&el.primitive) {
        let primitive_mesh = match el.primitive {
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
        };
        persistence
            .instanced_meshes
            .insert(el.primitive, InstancedEntity::new(context, &primitive_mesh));
    }

    let instanced = persistence
        .instanced_meshes
        .get_mut(&el.primitive)
        .expect("just checked it, will be there");
    instanced.transforms.push(entity_transform * el.transform);
    instanced.colors.push(Color {
        r: el.color.r,
        g: el.color.g,
        b: el.color.b,
        a: el.color.a,
    });
}

fn component_to_meshes<C: Component + Drawable + 'static>(
    persistence: &mut RenderPersistence,
    context: &Context,
    construct: &Construct,
) {
    for (element_id, component_with_drawables) in construct.world().component_iter::<C>() {
        // Get the world pose for this entity, to add draw transform local to this component.
        let world_pose = construct.entity_pose(&element_id);
        for el in component_with_drawables.drawables() {
            elements_to_render(persistence, context, &el, world_pose.transform())
        }
    }
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
            vec3(-5.0, 2.0, 1.5),
            vec3(0.0, 0.0, -0.5),
            vec3(0.0, 0.0, 1.0),
            degrees(45.0),
            0.1,
            1000.0,
        );
        let config = three_d::renderer::control::OrbitControlConfig {
            speed_orbit_horizontal: 0.1,
            speed_orbit_vertical: 0.1,
            speed_zoom: 2.0,
            speed_orbit_target_left: Some(0.1),
            speed_orbit_target_up: Some(0.1),
            ..Default::default()
        };

        let control = OrbitControl::new_with_config(config);
        // let mut control = FlyControl::new(0.1);

        let ambient_light =
            three_d::renderer::light::AmbientLight::new(&context, 0.1, Color::WHITE);
        let directional_light =
            DirectionalLight::new(&context, 1.5, Color::WHITE, &vec3(0.0, 0.0, -1.0));

        let mut persistence: RenderPersistence = Default::default();
        persistence.populate_default(&context);

        ConstructViewer {
            camera,
            context,
            ambient_light,
            directional_light,
            control,
            window,
            construct,
            limiter,
            persistence,
        }
    }

    // Consumes the viewer...
    fn view_loop(mut self) -> () {
        self.window.render_loop(move |mut frame_input: FrameInput| {
            while self.construct.elapsed_as_f64() < self.limiter.elapsed_as_f64() {
                let now = std::time::Instant::now();
                self.construct.update();
                if PRINT_DURATIONS {
                    println!(
                        "construct taken: {:1.8}, entities: {}",
                        now.elapsed().as_secs_f64(),
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
            let elements =
                Self::render_construct(&mut self.persistence, &self.context, &self.construct);

            self.persistence.update_instances();

            if PRINT_DURATIONS {
                println!("elements: {}", now.elapsed().as_secs_f64());
            }

            // Skip the ground plane in the shadow map, otherwise we get no resolution.
            self.directional_light.generate_shadow_map(
                2048,
                self.persistence
                    .instanced_meshes
                    .values()
                    .map(|x| &x.gm.geometry),
            );

            let now = std::time::Instant::now();

            screen.render(
                &self.camera,
                self.persistence.instanced_meshes.values().map(|x| x.gm()),
                &[&self.ambient_light, &self.directional_light],
            );

            screen.render(
                &self.camera,
                self.persistence.static_gms.iter(),
                &[&self.ambient_light, &self.directional_light],
            );

            self.persistence.reset_instances();

            if PRINT_DURATIONS {
                println!("render: {}", now.elapsed().as_secs_f64());
            }

            FrameOutput::default()
        });
    }

    pub fn render_construct(
        peristence: &mut RenderPersistence,
        context: &Context,
        construct: &Construct,
    ) {
        component_to_meshes::<display::tank_body::TankBody>(peristence, context, construct);
        component_to_meshes::<display::tank_tracks::TankTracks>(peristence, context, construct);

        component_to_meshes::<display::tank_turret::TankTurret>(peristence, context, construct);
        component_to_meshes::<display::tank_barrel::TankBarrel>(peristence, context, construct);
        component_to_meshes::<display::tank_bullet::TankBullet>(peristence, context, construct);

        component_to_meshes::<display::debug_box::DebugBox>(peristence, context, construct);
        component_to_meshes::<display::debug_sphere::DebugSphere>(peristence, context, construct);
    }
}

pub fn main() {
    let construct = Construct::new();
    let viewer = ConstructViewer::new(construct);

    // view loop consumes the viewer... :|
    viewer.view_loop();
}
