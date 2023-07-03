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

    // Add the generator for ids
    let generator_id = world.add_entity();
    world.add_component(generator_id, components::id_generator::IdGenerator::new());
}

pub fn add_systems(systems: &mut Systems) {
    // First, let the clock tick such that the time advances
    systems.add_system(Box::new(systems::clock::ClockSystem {}));
    // systems.add_system(Box::new(systems::record::Record {}));
    // Expire as many things as possible first, possible lightening work.
    systems.add_system(Box::new(systems::expiry_check::ExpiryCheck {}));

    // Then, run any game systems.
    systems.add_system(Box::new(systems::capture::Capture {}));
    systems.add_system(Box::new(
        systems::match_logic_king_of_the_hill::MatchLogicKingOfTheHill {},
    ));
    systems.add_system(Box::new(
        systems::match_logic_team_deathmatch::MatchLogicTeamDeathmatch {},
    ));
    systems.add_system(Box::new(
        systems::match_logic_domination::MatchLogicDomination {},
    ));
    systems.add_system(Box::new(
        systems::match_logic_time_limit::MatchLogicTimeLimit {},
    ));
    systems.add_system(Box::new(
        systems::match_logic_finished::MatchLogicFinished {},
    ));

    // Physics systems
    systems.add_system(Box::new(
        systems::kinematics_differential_drive::KinematicsDifferentialDrive {},
    ));
    systems.add_system(Box::new(
        systems::kinematics_tricycle::KinematicsTricycle {},
    ));
    systems.add_system(Box::new(
        systems::acceleration_velocity::AccelerationVelocity {},
    ));
    systems.add_system(Box::new(systems::velocity_pose::VelocityPose {}));

    // Update performs revolute integration, pose and velocity set.
    systems.add_system(Box::new(systems::revolute_update::RevoluteUpdate {}));
    // systems.add_system(Box::new(systems::revolute_pose::RevolutePose {}));
    // systems.add_system(Box::new(systems::revolute_velocity::RevoluteVelocity {}));

    // Projectile system handling, hit calculation, impact processing
    systems.add_system(Box::new(systems::projectile_hit::ProjectileHit {}));
    systems.add_system(Box::new(systems::process_impact::ProcessImpact {}));
    // ProcessHitBy MUST go after the hit calculation.
    systems.add_system(Box::new(systems::process_hit_by::ProcessHitBy {}));

    // Next, determine the health of any unit, mark them as destroyed if applicable.
    systems.add_system(Box::new(systems::health_check::HealthCheck {}));

    // Destroy anything marked as destroyed by the health check.
    systems.add_system(Box::new(systems::destroy::Destroy {}));

    // Coloring / display systems, they don't really matter when they go.
    // systems.add_system(Box::new(systems::health_tank_body::HealthTankBody {}));
    systems.add_system(Box::new(systems::team_color_body::TeamColorBody {}));
    systems.add_system(Box::new(systems::health_bar_update::HealthBarUpdate {}));
    systems.add_system(Box::new(systems::display_tank_tracks::DisplayTankTracks {}));
    systems.add_system(Box::new(
        systems::display_tricycle_wheels::DisplayTricycleWheels {},
    ));
    systems.add_system(Box::new(
        systems::display_capture_flag::DisplayCaptureFlag {},
    ));

    systems.add_system(Box::new(systems::victory_effect::VictoryEffect {}));

    // Update function positions.
    systems.add_system(Box::new(systems::function_pose::FunctionPose {}));
    systems.add_system(Box::new(systems::timed_function::TimedFunction {}));

    // Calculate the radio messagse.
    systems.add_system(Box::new(systems::radio_transmission::RadioTransmission {}));

    // Calculate the radar hits
    systems.add_system(Box::new(systems::radar_scan::RadarScan {}));
    // Run the unit controllers
    systems.add_system(Box::new(systems::unit_control::UnitControl {}));

    // After the unit controller, check if any controllers errored.
    systems.add_system(Box::new(
        systems::unit_controller_error_check::UnitControllerErrorCheck {},
    ));
    // Run another destroy check, it's cheap and this ensures units that are destroyed because
    // their controller failed don't fire anymore with their dying breath.
    systems.add_system(Box::new(systems::destroy::Destroy {}));

    // Shoot any cannons
    systems.add_system(Box::new(systems::cannon_trigger::CannonTrigger {}));
    systems.add_system(Box::new(systems::gun_battery_trigger::GunBatteryTrigger {}));
}
