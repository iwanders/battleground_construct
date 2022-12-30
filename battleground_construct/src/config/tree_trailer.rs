use crate::components;
use crate::display;
use crate::units;

use units::tank::{spawn_tank, TankSpawnConfig};

// a535e66f594aa51758273bf0f98ec6f2e3f4381f is the last commit before 'snow' was removed.

pub fn populate_tree_trailer(construct: &mut crate::Construct) {
    let world = &mut construct.world;
    let systems = &mut construct.systems;
    use engine::prelude::*;

    super::default::add_components(world);
    super::default::add_systems(systems);

    use components::function_pose::FunctionPose;
    use components::pose::Pose;
    use display::Color;

    use crate::display::primitives::*;
    use cgmath::Deg;

    fn add_tree(world: &mut engine::World, x: f32, y: f32) -> engine::EntityId {
        let decoration = true;
        let decoration_radius = 0.05;
        let decoration_colors = [Color::RED, Color::GREEN, Color::BLUE];

        let emissive_colors: Vec<Color> = decoration_colors
            .iter()
            .copied()
            .map(|em| Color {
                r: em.r.saturating_sub(128),
                g: em.g.saturating_sub(128),
                b: em.b.saturating_sub(128),
                a: em.a.saturating_sub(128),
            })
            .collect();

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

        let make_decoration =
            |radius: f32,
             height: f32,
             count: usize,
             seed_offset: f32,
             elements: &mut display::debug_elements::DebugElements| {
                if decoration {
                    let o = (seed_offset + 100.0) as usize;
                    for i in 0..count {
                        let v =
                            ((i as f32 / count as f32) * std::f32::consts::PI * 2.0) + seed_offset;
                        let y = v.sin() * radius;
                        let z = v.cos() * radius;
                        let color_index = (i + o) % decoration_colors.len();
                        let material = display::primitives::Material::FlatMaterial(FlatMaterial {
                            color: decoration_colors[color_index],
                            emissive: emissive_colors[color_index],
                            is_emissive: true,
                            ..Default::default()
                        });
                        elements.add_element(Element {
                            transform: Mat4::from_translation(Vec3::new(height, y, z)),
                            primitive: Primitive::Sphere(Sphere {
                                radius: decoration_radius,
                            }),
                            material: material,
                        });
                    }
                }
            };
        make_decoration(0.65, 0.5 - 0.20 - decoration_radius, 12, x, &mut elements);

        elements.add_element(Element {
            transform: Mat4::from_translation(Vec3::new(1.0 - 0.20, 0.0, 0.0)),
            primitive: Primitive::Cone(Cone {
                height: 0.75,
                radius: 0.5,
            }),
            material: Color::rgb(0, 90, 0).into(),
        });
        make_decoration(
            0.5,
            1.0 - 0.20 - decoration_radius,
            7,
            y + 1.0,
            &mut elements,
        );

        elements.add_element(Element {
            transform: Mat4::from_translation(Vec3::new(1.5 - 0.20, 0.0, 0.0)),
            primitive: Primitive::Cone(Cone {
                height: 0.5,
                radius: 0.3,
            }),
            material: Color::rgb(0, 90, 0).into(),
        });
        make_decoration(
            0.3,
            1.5 - 0.20 - decoration_radius,
            5,
            x + 2.0,
            &mut elements,
        );

        elements.add_element(Element {
            transform: Mat4::from_translation(Vec3::new(0.0, 0.0, 0.0)),
            primitive: Primitive::Cylinder(Cylinder {
                height: 1.0,
                radius: 0.15,
            }),
            material: Color::rgb(50, 24, 0).into(),
        });
        world.add_component(tree, Pose::from_xyz(x, y, 0.0).rotated_angle_y(Deg(-90.0)));
        world.add_component(tree, elements);
        tree
    }

    let _tree1 = add_tree(world, -6.0, -3.75);
    let _tree2 = add_tree(world, -6.0, -4.5);
    let _tree3 = add_tree(world, -5.5, -4.75);
    let tree4 = add_tree(world, -4.8, -5.0);
    let tree5 = add_tree(world, -4.0, -5.25);
    let _tree6 = add_tree(world, -3.0, -5.5);
    let _tree7 = add_tree(world, -2.0, -5.75);

    /*
    let particle_effect_id = components::id_generator::generate_id(world);
    let thingy = world.add_entity();
    let mut destructor = display::deconstructor::Deconstructor::new(particle_effect_id);
    destructor.add_element::<display::debug_elements::DebugElements>(tree1, world);
    world.add_component(thingy, destructor);
    world.add_component(thingy, components::expiry::Expiry::lifetime(50.0));
    */

    /*
    let thingy = world.add_entity();
    let particle_effect_id = components::id_generator::generate_id(world);
    world.add_component(thingy, Pose::from_xyz(-5.0, -5.0, 5.0));
    world.add_component(
        thingy,
        display::particle_emitter::ParticleEmitter::snow(particle_effect_id, 0.03, Color::WHITE),
    );*/

    fn generate_pose_function(
        controls: Vec<(f32, cgmath::Vector3<f32>)>,
    ) -> Box<dyn Fn(f32) -> Pose> {
        Box::new(move |t| {
            for w in controls[..].windows(2) {
                let c = w[0];
                let n = w[1];
                if c.0 < t && t <= n.0 {
                    // lerp in the vector.
                    let ratio = (t - c.0) / (n.0 - c.0);
                    let p = c.1 + (n.1 - c.1) * ratio;
                    return Pose::from_xyz(p.x, p.y, p.z);
                }
            }
            let f = controls.last().unwrap().1;
            Pose::from_xyz(f.x, f.y, f.z)
        })
    }

    if true {
        use cgmath::vec3;
        let camera_path = vec![
            (0.0, vec3(-8.0, -7.0, 5.0)),
            (5.0, vec3(-8.0, -7.0, 5.0)),
            (15.0, vec3(-8.0, -7.0, 1.0)),
            // (15.0, vec3(-2.0, -5.0, 0.0)),
        ];

        let camera_direction = camera_path
            .iter()
            .map(|(t, p)| (*t, p + vec3(1.0, 0.5, 0.0)))
            .collect();

        let camera_target = world.add_entity();
        world.add_component(camera_target, Pose::from_xyz(0.0, 0.0, 0.0));
        world.add_component(
            camera_target,
            components::camera_target::CameraTarget::new(),
        );
        world.add_component(
            camera_target,
            FunctionPose::new(generate_pose_function(camera_direction)),
        );

        let camera = world.add_entity();
        world.add_component(camera, Pose::from_xyz(-1.0, -1.0, 0.0));
        world.add_component(camera, components::camera_position::CameraPosition::new());
        world.add_component(
            camera,
            FunctionPose::new(generate_pose_function(camera_path)),
        );
    }

    let t_blast = 20.0;
    use components::timed_function_trigger::TimedFunctionTrigger;
    let timed_destructor = world.add_entity();
    world.add_component(
        timed_destructor,
        TimedFunctionTrigger::new(t_blast, None, move |_: _, world: &mut World| {
            let particle_effect_id = components::id_generator::generate_id(world);
            let thingy = world.add_entity();
            let mut destructor = display::deconstructor::Deconstructor::new(particle_effect_id);
            destructor.add_element::<display::debug_elements::DebugElements>(tree4, world);
            destructor.add_element::<display::debug_elements::DebugElements>(tree5, world);
            use crate::util::cgmath::ToHomogenous;
            // directed blast from left
            destructor.add_impact(cgmath::vec3(-4.0, -4.0, 1.0).to_h(), 3.5);
            destructor.add_impact(cgmath::vec3(-5.0, -3.5, 1.0).to_h(), 4.5);
            // In the trees;
            destructor.add_impact(cgmath::vec3(-4.8, -5.0, 1.0).to_h(), 0.1);
            destructor.add_impact(cgmath::vec3(-4.0, -5.25, 1.0).to_h(), 0.1);
            world.add_component(thingy, destructor);
            world.add_component(thingy, components::expiry::Expiry::lifetime(50.0));
            world.remove_entity(tree4);
            world.remove_entity(tree5);
        }),
    );

    // Spawn the robot at the correct rotation, but incorrect position, such that it is invissible.
    let main_tank = spawn_tank(
        world,
        TankSpawnConfig {
            x: -8.0,
            y: -8.0,
            yaw: (-110.0) / 360.0 * 2.0 * std::f32::consts::PI,
            controller: Box::new(unit_control_builtin::idle::Idle {}),
            ..Default::default()
        },
    );
    let main_tank_entities = world
        .component::<crate::units::tank::UnitTank>(main_tank)
        .unwrap()
        .clone();
    let base_entity = main_tank_entities.base_entity;
    let turret_entity = main_tank_entities.turret_entity;
    let barrel_entity = main_tank_entities.barrel_entity;
    let muzzle_entity = main_tank_entities.muzzle_entity;

    let timed_spawn = world.add_entity();
    world.add_component(
        timed_spawn,
        TimedFunctionTrigger::new(t_blast, None, move |_: _, world: &mut World| {
            // Teleport the robot to the start position.
            {
                let mut g = world
                    .component_mut::<components::pose::Pose>(base_entity)
                    .unwrap();
                g.transform_mut().w.x = -3.5;
                g.transform_mut().w.y = -3.7;
            }
            // Set the drive to on.
            let mut d = world
                .component_mut::<components::differential_drive_base::DifferentialDriveBase>(
                    base_entity,
                )
                .expect("diff drive not on expected entity.");
            d.set_velocities(0.5, 0.5);
        }),
    );

    let modify_revolute_speed = |world: &mut World, time: f32, entity: EntityId, vel: f32| {
        let modify_rev = world.add_entity();
        world.add_component(
            modify_rev,
            TimedFunctionTrigger::new(time, None, move |_: _, world: &mut World| {
                let mut d = world
                    .component_mut::<components::revolute::Revolute>(entity)
                    .expect("revolute should be here");
                d.set_velocity(vel);
            }),
        );
    };

    modify_revolute_speed(world, t_blast + 2.0, turret_entity, -0.3);
    modify_revolute_speed(world, t_blast + 5.0, turret_entity, 0.0);

    modify_revolute_speed(world, t_blast + 3.0, barrel_entity, -0.10);
    modify_revolute_speed(world, t_blast + 5.0, barrel_entity, 0.0);

    let stop_moving = world.add_entity();
    world.add_component(
        stop_moving,
        TimedFunctionTrigger::new(t_blast + 4.0, None, move |_: _, world: &mut World| {
            let mut d = world
                .component_mut::<components::differential_drive_base::DifferentialDriveBase>(
                    base_entity,
                )
                .unwrap();
            d.set_velocities(0.0, 0.0);
        }),
    );

    let set_cannon_firing = |world: &mut World, time: f32| {
        let modify_rev = world.add_entity();
        world.add_component(
            modify_rev,
            TimedFunctionTrigger::new(time, None, move |_: _, world: &mut World| {
                let mut d = world
                    .component_mut::<components::cannon::Cannon>(muzzle_entity)
                    .expect("cannon should be here");
                d.trigger();
            }),
        );
    };
    set_cannon_firing(world, t_blast + 5.5);
    set_cannon_firing(world, t_blast + 5.6);
}
