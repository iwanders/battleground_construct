use crate::components;
use crate::control;
use crate::display;
use crate::units;

use units::tank::{spawn_tank, TankSpawnConfig};

// This held the dev playground for the longest time.
pub fn populate_dev_world(construct: &mut crate::Construct) {
    let world = &mut construct.world;
    let systems = &mut construct.systems;

    super::default::add_components(world);
    super::default::add_systems(systems);

    use components::function_pose::FunctionPose;
    use components::pose::Pose;
    use display::flag::Flag;
    use display::Color;

    let flag_id = world.add_entity();
    world.add_component(flag_id, components::pose::Pose::from_xyz(-1.0, -1.0, 0.0));
    world.add_component(flag_id, display::flag::Flag::new());

    let flag_id = world.add_entity();
    world.add_component(flag_id, components::pose::Pose::from_xyz(1.0, -1.0, 0.0));
    world.add_component(flag_id, Flag::from_scale_color(0.5, Color::GREEN));

    let particle_id = world.add_entity();
    world.add_component(particle_id, Pose::from_xyz(-1.0, -1.0, 0.0));
    world.add_component(particle_id, Flag::from_scale_color(0.5, Color::MAGENTA));
    world.add_component(
        particle_id,
        FunctionPose::new(|t| Pose::from_xyz(t.sin() - 2.0, t.cos() + 2.0, t.sin() + 1.5)),
    );

    let particle_effect_id = components::id_generator::generate_id(world);
    world.add_component(
        particle_id,
        display::particle_emitter::ParticleEmitter::bullet_trail(
            particle_effect_id,
            0.05,
            Color::WHITE,
        ),
    );

    let main_tank = spawn_tank(
        world,
        TankSpawnConfig {
            x: 0.0,
            y: 0.0,
            yaw: 0.0,
            controller: Box::new(control::tank_swivel_shoot::TankSwivelShoot {}),
            // controller: control::dynamic_load_control::DynamicLoadControl::new(
            // "../target/release/libvehicle_control_wasm.so",
            // )
            // .expect("should succeed"),
            ..Default::default()
        },
    );
    /**/

    let _radar_tank = spawn_tank(
        world,
        TankSpawnConfig {
            x: -2.0,
            y: -3.0,
            yaw: 0.0,
            // controller: Box::new(control::tank_swivel_shoot::TankSwivelShoot {}),
            controller: Box::new(control::radar_draw::RadarDrawControl {}),
            ..Default::default()
        },
    );

    let mut tank_entities = vec![];
    {
        let g = world
            .component::<components::group::Group>(main_tank)
            .unwrap();
        for part_entity in g.entities().iter().copied() {
            tank_entities.push(part_entity);
        }
    }

    let particle_effect_id = components::id_generator::generate_id(world);
    let thingy = world.add_entity();
    let mut destructor = display::deconstructor::Deconstructor::new(particle_effect_id);
    for e in tank_entities.iter() {
        destructor.add_element::<display::tank_body::TankBody>(*e, world);
        destructor.add_element::<display::tank_turret::TankTurret>(*e, world);
        destructor.add_element::<display::tank_barrel::TankBarrel>(*e, world);
        // destructor.add_element::<display::flag::Flag>(*e, world);
    }

    // Add a sphere to the initial destructor.
    let sphere = world.add_entity();
    world.add_component(sphere, display::debug_sphere::DebugSphere::new());
    world.add_component(sphere, Pose::from_xyz(0.0, 0.0, 1.0));
    destructor.add_element::<display::debug_sphere::DebugSphere>(sphere, world);
    world.remove_entity(sphere); // but not visualise it.

    // world.add_component(thingy, Pose::from_xyz(0.0, 0.0, 0.0));
    world.add_component(thingy, destructor);
    world.add_component(thingy, components::expiry::Expiry::lifetime(50.0));

    for x in 1..5 {
        for y in -2..2 {
            if !(x == 4 && y == -2) {
                // continue;
            }
            spawn_tank(
                world,
                TankSpawnConfig {
                    x: x as f32 * 2.0 + 2.0,
                    y: y as f32 * 3.0 - 2.5,
                    yaw: std::f32::consts::PI / 2.0,
                    controller: Box::new(
                        control::diff_drive_forwards_backwards::DiffDriveForwardsBackwardsControl {
                            velocities: (1.0, 1.0),
                            last_flip: 0.0,
                            duration: 5.0,
                        },
                    ),
                    ..Default::default()
                },
            );
        }
    }

    use crate::display::primitives::*;
    use cgmath::Deg;

    fn add_tree(world: &mut engine::World, x: f32, y: f32) -> engine::EntityId {
        let tree = world.add_entity();
        let mut elements = display::debug_elements::DebugElements::new();
        elements.add_element(Element {
            transform: Mat4::from_translation(Vec3::new(0.5 - 0.20, 0.0, 0.0)),
            primitive: Primitive::Cone(Cone {
                height: 1.0,
                radius: 0.65,
            }),
            material: Color::rgb(0, 90, 0).into(),
        });
        elements.add_element(Element {
            transform: Mat4::from_translation(Vec3::new(1.0 - 0.20, 0.0, 0.0)),
            primitive: Primitive::Cone(Cone {
                height: 0.75,
                radius: 0.5,
            }),
            material: Color::rgb(0, 90, 0).into(),
        });
        elements.add_element(Element {
            transform: Mat4::from_translation(Vec3::new(1.5 - 0.20, 0.0, 0.0)),
            primitive: Primitive::Cone(Cone {
                height: 0.5,
                radius: 0.3,
            }),
            material: Color::rgb(0, 90, 0).into(),
        });
        elements.add_element(Element {
            transform: Mat4::from_translation(Vec3::new(0.0, 0.0, 0.0)),
            primitive: Primitive::Cylinder(Cylinder {
                height: 1.0,
                radius: 0.15,
            }),
            material: Color::rgb(50, 24, 0).into(),
        });
        world.add_component(
            tree,
            Pose::from_xyz(x, y, 0.0).rotated_angle_y(Deg(-90.0)),
        );
        world.add_component(tree, elements);
        tree
    }

    let tree1 = add_tree(world, -6.0, -4.75);
    let tree1 = add_tree(world, -5.0, -5.0);
    let tree2 = add_tree(world, -3.0, -5.5);
    let tree2 = add_tree(world, -4.0, -5.25);
    let tree2 = add_tree(world, -2.0, -5.75);

    let particle_effect_id = components::id_generator::generate_id(world);
    let thingy = world.add_entity();
    let mut destructor = display::deconstructor::Deconstructor::new(particle_effect_id);
    destructor.add_element::<display::debug_elements::DebugElements>(tree1, world);
    world.add_component(thingy, destructor);
    world.add_component(thingy, components::expiry::Expiry::lifetime(50.0));


    let thingy = world.add_entity();
    let particle_effect_id = components::id_generator::generate_id(world);
    world.add_component(thingy, Pose::from_xyz(-5.0, -5.0, 5.0));
    world.add_component(
        thingy,
        display::particle_emitter::ParticleEmitter::snow(
            particle_effect_id,
            0.03,
            Color::WHITE,
        ),
    );

    // world.remove_entity(tree);
}
