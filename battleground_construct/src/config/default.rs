use crate::components;
use crate::systems;
use engine::*;

pub fn add_components(world: &mut World) {
    // The clock... is kinda important.
    let clock_id = world.add_entity();
    world.add_component(clock_id, components::clock::Clock::new());

    // Add the floor, seems relevant as well.
    let floor_id = world.add_entity();
    world.add_component(floor_id, components::pose::Pose::new());
    world.add_component(floor_id, components::hit_plane::HitPlane::new());
}

pub fn add_systems(systems: &mut Systems) {
    systems.add_system(Box::new(systems::clock::ClockSystem {}));
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
    systems.add_system(Box::new(systems::projectile_hit::ProjectileHit {}));
    systems.add_system(Box::new(systems::process_impact::ProcessImpact {}));
    // Must go after the hit calculation.
    systems.add_system(Box::new(systems::process_hit_by::ProcessHitBy {}));

    systems.add_system(Box::new(systems::health_check::HealthCheck {}));
    systems.add_system(Box::new(systems::destroy::Destroy {}));
    // All handling of hits done with the projectiles still present.
    // systems.add_system(Box::new(systems::health_tank_body::HealthTankBody {}));
    systems.add_system(Box::new(
        systems::team_color_tank_body::TeamColorTankBody {},
    ));
    systems.add_system(Box::new(systems::display_tank_tracks::DisplayTankTracks {}));
    systems.add_system(Box::new(systems::function_pose::FunctionPose {}));
    systems.add_system(Box::new(systems::expiry_check::ExpiryCheck {}));
    systems.add_system(Box::new(systems::unit_control::UnitControl {}));
}
