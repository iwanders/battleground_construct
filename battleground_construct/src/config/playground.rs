use crate::components;
use crate::control;
use crate::display;
use crate::systems;
use crate::vehicles;
// use crate::ClockSystem;
use engine::{Systems, World};

use components::clock::{Clock, ClockSystem};
use vehicles::tank::{spawn_tank, TankSpawnConfig};

// This held the dev playground for the longest time.
pub fn populate_dev_world(world_systems: (&mut World, &mut Systems)) {
    let world = world_systems.0;
    let systems = world_systems.1;
    let clock_id = world.add_entity();
    world.add_component(clock_id, Clock::new());

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
    world.add_component(
        particle_id,
        display::particle_emitter::ParticleEmitter::bullet_trail(particle_id, 0.05, Color::WHITE),
    );

    // Add the floor.
    let floor_id = world.add_entity();
    world.add_component(floor_id, components::pose::Pose::new());
    world.add_component(floor_id, components::hit_plane::HitPlane::new());

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

    let thingy = world.add_entity();
    let mut destructor = display::deconstructor::Deconstructor::new(thingy);
    for e in tank_entities.iter() {
        destructor.add_element::<display::tank_body::TankBody>(*e, &world);
        destructor.add_element::<display::tank_turret::TankTurret>(*e, &world);
        destructor.add_element::<display::tank_barrel::TankBarrel>(*e, &world);
        // destructor.add_element::<display::flag::Flag>(*e, &world);
    }

    // Add a sphere to the initial destructor.
    let sphere = world.add_entity();
    world.add_component(sphere, display::debug_sphere::DebugSphere::new());
    world.add_component(sphere, Pose::from_xyz(0.0, 0.0, 1.0));
    destructor.add_element::<display::debug_sphere::DebugSphere>(sphere, &world);
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
                },
            );
        }
    }

    systems.add_system(Box::new(ClockSystem {}));
    systems.add_system(Box::new(
        systems::kinematics_differential_drive::KinematicsDifferentialDrive {},
    ));
    systems.add_system(Box::new(
        systems::acceleration_velocity::AccelerationVelocity {},
    ));
    systems.add_system(Box::new(systems::velocity_pose::VelocityPose {}));
    // systems.add_system(Box::new(systems::revolute_pose::RevolutePose {}));
    systems.add_system(Box::new(systems::revolute_velocity::RevoluteVelocity {}));
    systems.add_system(Box::new(systems::radar_scan::RadarScan {}));
    systems.add_system(Box::new(systems::cannon_trigger::CannonTrigger {}));
    // systems.add_system(Box::new(systems::projectile_floor::ProjectileFloor {}));
    systems.add_system(Box::new(systems::projectile_hit::ProjectileHit {}));
    systems.add_system(Box::new(systems::process_impact::ProcessImpact {}));
    // Must go after the hit calculation.
    systems.add_system(Box::new(systems::process_hit_by::ProcessHitBy {}));

    systems.add_system(Box::new(systems::health_check::HealthCheck {}));
    systems.add_system(Box::new(systems::destroy::Destroy {}));
    // All handling of hits done with the projectiles still present.
    // systems.add_system(Box::new(systems::health_tank_body::HealthTankBody {}));
    systems.add_system(Box::new(systems::display_tank_tracks::DisplayTankTracks {}));
    systems.add_system(Box::new(systems::function_pose::FunctionPose {}));
    systems.add_system(Box::new(systems::expiry_check::ExpiryCheck {}));
    systems.add_system(Box::new(systems::vehicle_control::VehicleControl {}));
}
