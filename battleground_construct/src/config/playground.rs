#![allow(unused_variables)]
#![allow(unused_imports)]

use crate::components;
use crate::control;
use crate::display;
use crate::units;
use engine::EntityId;
use unit_control_builtin;

use crate::util::cgmath::prelude::*;
use crate::util::cgmath::{Mat4, Vec3};

use units::artillery::{artillery_battery_config, spawn_artillery, ArtillerySpawnConfig};
use units::tank::{spawn_tank, TankSpawnConfig};

// This held the dev playground for the longest time.
pub fn populate_dev_world(construct: &mut crate::Construct) {
    let world = &mut construct.world;
    let systems = &mut construct.systems;

    // super::default::add_components(world);
    // super::default::add_systems(systems);

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
            controller: Box::new(unit_control_builtin::tank_swivel_shoot::TankSwivelShoot::new()),
            // controller: control::dynamic_load_control::DynamicLoadControl::new(
            // "../target/release/libvehicle_control_wasm.so",
            // )
            // .expect("should succeed"),
            ..Default::default()
        },
    );
    /**/
    let radar_tank = spawn_tank(
        world,
        TankSpawnConfig {
            x: -2.0,
            y: -3.0,
            yaw: 0.0,
            // controller: Box::new(unit_control_builtin::tank_swivel_shoot::TankSwivelShoot::new()),
            controller: Box::new(control::radar_draw::RadarDrawControl {}),
            ..Default::default()
        },
    );

    let fw_backwards = Box::new(
        unit_control_builtin::diff_drive_forwards_backwards::DiffDriveForwardsBackwardsControl {
            velocities: (1.0, 1.0),
            last_flip: -2.5,
            duration: 5.0,
        },
    );

    let artillery = spawn_artillery(
        world,
        ArtillerySpawnConfig {
            x: 0.0,
            y: 3.0,
            // yaw: std::f32::consts::PI / 2.0,
            yaw: 0.0,
            // controller: Box::new(unit_control_builtin::tank_swivel_shoot::TankSwivelShoot::new()),
            // controller: Box::new(control::radar_draw::RadarDrawControl {}),
            controller: Box::new(unit_control_builtin::idle::Idle {}),
            // controller: fw_backwards,
            ..Default::default()
        },
    );

    let mut destroy_entities = vec![];
    {
        let g = world
            .component::<components::group::Group>(artillery)
            .unwrap();
        for part_entity in g.entities().iter().copied() {
            destroy_entities.push(part_entity);
        }
    }

    let particle_effect_id = components::id_generator::generate_id(world);
    let thingy = world.add_entity();
    let mut destructor = display::deconstructor::Deconstructor::new(particle_effect_id);
    for e in destroy_entities.iter() {
        destructor.add_element::<display::tank_body::TankBody>(*e, world);
        destructor.add_element::<display::tank_turret::TankTurret>(*e, world);
        destructor.add_element::<display::tank_barrel::TankBarrel>(*e, world);

        destructor.add_element::<display::artillery_body::ArtilleryBody>(*e, world);
        destructor.add_element::<display::artillery_turret::ArtilleryTurret>(*e, world);
        destructor.add_element::<display::artillery_barrel::ArtilleryBarrel>(*e, world);
        // destructor.add_element::<display::flag::Flag>(*e, world);
    }

    /*
    // Add a sphere to the initial destructor.
    let sphere = world.add_entity();
    world.add_component(sphere, display::debug_sphere::DebugSphere::new());
    world.add_component(sphere, Pose::from_xyz(0.0, 0.0, 1.0));
    destructor.add_element::<display::debug_sphere::DebugSphere>(sphere, world);
    world.remove_entity(sphere); // but not visualise it.

    // world.add_component(thingy, Pose::from_xyz(0.0, 0.0, 0.0));
    world.add_component(thingy, destructor);
    world.add_component(thingy, components::expiry::Expiry::lifetime(50.0));
    */

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
                        unit_control_builtin::diff_drive_forwards_backwards::DiffDriveForwardsBackwardsControl {
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

    // Add a cannon projectile emitter.
    use crate::components::timed_function_trigger::TimedFunctionTrigger;
    use cgmath::Deg;
    let static_cannon = world.add_entity();
    world.add_component(
        static_cannon,
        components::pose::Pose::from_xyz(2.0, 4.0, 1.0).rotated_angle_y(Deg(-45.0)),
    );
    let cannon = components::cannon::Cannon::new(components::cannon::CannonConfig {
        fire_effect: std::rc::Rc::new(crate::units::tank::cannon_function),
        reload_time: 1.0,
    });
    world.add_component(static_cannon, cannon);
    world.add_component(static_cannon, display::debug_box::DebugBox::cube(0.1));

    let cannon_shoot_interval = 1.0;
    world.add_component(
        static_cannon,
        TimedFunctionTrigger::new(
            0.0,
            Some(cannon_shoot_interval),
            move |_: _, world: &mut engine::World| {
                let mut cannon = world
                    .component_mut::<components::cannon::Cannon>(static_cannon)
                    .unwrap();
                cannon.trigger();
            },
        ),
    );

    // Add a artillery gun battery emitter.
    let static_artillery_barrel = world.add_entity();
    world.add_component(
        static_artillery_barrel,
        components::pose::Pose::from_xyz(2.0, 6.0, 1.0).rotated_angle_y(Deg(-45.0)),
    );
    world.add_component(
        static_artillery_barrel,
        display::artillery_barrel::ArtilleryBarrel::new(),
    );

    use battleground_unit_control::units::artillery::ARTILLERY_DIM_BARREL_TO_MUZZLE_X;
    let static_artillery_muzzle_entity = world.add_entity();
    world.add_component(
        static_artillery_muzzle_entity,
        components::parent::Parent::new(static_artillery_barrel),
    );
    world.add_component(
        static_artillery_muzzle_entity,
        components::pose::PreTransform::from_translation(Vec3::new(
            ARTILLERY_DIM_BARREL_TO_MUZZLE_X,
            0.0,
            0.0,
        )),
    );
    world.add_component(
        static_artillery_muzzle_entity,
        display::debug_box::DebugBox::cube(0.1),
    );
    let mut gun_battery = components::gun_battery::GunBattery::new(artillery_battery_config());
    gun_battery.set_trigger(true);
    world.add_component(static_artillery_muzzle_entity, gun_battery);
    world.add_component(
        static_artillery_muzzle_entity,
        display::debug_box::DebugBox::cube(0.1),
    );

    let artillery_target = spawn_artillery(
        world,
        ArtillerySpawnConfig {
            x: 15.0,
            y: 6.5,
            yaw: std::f32::consts::PI / 2.0,
            controller: Box::new(
                unit_control_builtin::diff_drive_forwards_backwards::DiffDriveForwardsBackwardsControl {
                    velocities: (1.0, 1.0),
                    last_flip: 0.0,
                    duration: 5.0,
                },
            ),
            ..Default::default()
        },
    );

    // Spawn two teams.
    let team_red_entity = world.add_entity();
    let red_team_id = components::id_generator::generate_id(world);
    let red_team = components::team::Team::new(red_team_id, "red", Color::RED);
    let red_team_id = red_team.id();
    world.add_component(team_red_entity, red_team);
    let team_blue_id = components::id_generator::generate_id(world);
    let team_blue_entity = world.add_entity();
    let blue_team = components::team::Team::new(team_blue_id, "blue", Color::BLUE);
    let team_blue_id = blue_team.id();
    world.add_component(team_blue_entity, blue_team);

    // Spawn a capturable flag.
    let mut flag_config = crate::units::capturable_flag::CapturableFlagConfig {
        x: -3.0,
        y: 5.0,
        yaw: 0.0,
        radius: 3.0,
        capture_speed: 1.0,
        initial_owner: Some(red_team_id),
        ..Default::default()
    };
    crate::units::capturable_flag::spawn_capturable_flag(world, flag_config);
    flag_config.initial_owner = Some(team_blue_id);
    flag_config.x += -7.0;
    crate::units::capturable_flag::spawn_capturable_flag(world, flag_config);
    flag_config.x += -7.0;
    flag_config.initial_owner = None;
    crate::units::capturable_flag::spawn_capturable_flag(world, flag_config);

    spawn_tank(
        world,
        TankSpawnConfig {
            x: -6.5,
            y: 5.0,
            yaw: 0.0,
            // controller: Box::new(control::tank_swivel_shoot::TankSwivelShoot::new()),
            // controller: Box::new(unit_control_builtin::idle::Idle{}),
            controller: Box::new(
                unit_control_builtin::diff_drive_forwards_backwards::DiffDriveForwardsBackwardsControl {
                    velocities: (0.75, 1.0),
                    last_flip: -1.5,
                    duration: 3.0,
                },
            ),
            ..Default::default()
        },
    );

    // Add a firework emitter.
    // use crate::components::timed_function_trigger::TimedFunctionTrigger;
    let fireworks_shoot_interval = 3.0;
    let fireworks_shoorter = world.add_entity();
    world.add_component(
        fireworks_shoorter,
        TimedFunctionTrigger::new(
            0.0,
            Some(fireworks_shoot_interval),
            move |_: _, world: &mut engine::World| {
                let firework_entity = world.add_entity();
                world.add_component(
                    firework_entity,
                    components::pose::Pose::from_xyz(-2.0, 4.0, 1.0),
                );
                world.add_component(
                    firework_entity,
                    components::velocity::Velocity::from_linear(Vec3::new(0.0, 1.0, 8.0)),
                );
                world.add_component(
                    firework_entity,
                    components::acceleration::Acceleration::gravity(),
                );
                // world.add_component(firework_entity, display::debug_box::DebugBox::cube(0.1));

                let effect_id = components::id_generator::generate_id(world);
                world.add_component(
                    firework_entity,
                    crate::display::particle_emitter::ParticleEmitter::bullet_trail(
                        effect_id,
                        0.05,
                        crate::display::Color::rgb(255, 0, 0),
                    ),
                );

                world.add_component(
                    firework_entity,
                    TimedFunctionTrigger::after(
                        1.0,
                        |firework_entity: EntityId, world: &mut engine::World| {
                            // world.remove_entity(firework_entity);
                            world.remove_component::<components::velocity::Velocity>(
                                firework_entity,
                            );

                            let effect_id = components::id_generator::generate_id(world);
                            world.add_component(
                                firework_entity,
                                crate::display::particle_emitter::ParticleEmitter::firework(
                                    effect_id,
                                    2.0,
                                    crate::display::Color::rgb(255, 0, 0),
                                ),
                            );
                            world.add_component(
                                firework_entity,
                                crate::components::expiry::Expiry::lifetime(5.0),
                            );
                        },
                    ),
                );
            },
        ),
    );
}
