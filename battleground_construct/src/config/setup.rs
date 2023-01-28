use super::cli::Setup;
use super::default;
use super::specification;
use crate::components;
use crate::systems;
use crate::units;
use crate::Construct;
use battleground_unit_control::UnitControl;

#[derive(Debug, Clone)]
pub struct SetupError(pub String);
impl SetupError {
    pub fn new(text: &str) -> Self {
        SetupError(text.to_owned())
    }
}

impl std::error::Error for SetupError {}

impl std::fmt::Display for SetupError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub fn setup(config: &Setup) -> Result<Construct, Box<dyn std::error::Error>> {
    match config {
        Setup::Scenario(scenario) => setup_scenario(scenario),
        Setup::Play(path) => setup_playback_path(path),
        Setup::PlayBytes(bytes) => setup_playback_slice(bytes),
    }
}

pub fn setup_playback_slice(data: &[u8]) -> Result<Construct, Box<dyn std::error::Error>> {
    setup_playback_common(components::recording::Recording::load_slice(data)?)
}

pub fn setup_playback_path(path: &str) -> Result<Construct, Box<dyn std::error::Error>> {
    setup_playback_common(components::recording::Recording::load_file(path)?)
}

fn setup_playback_common(
    recorder: components::recording::Recording,
) -> Result<Construct, Box<dyn std::error::Error>> {
    let mut construct = Construct::new();
    let world = &mut construct.world;
    let systems = &mut construct.systems;

    // create the record component. then add that.
    let recorder_entity = world.add_entity();
    world.add_component(recorder_entity, recorder);

    systems.add_system(Box::new(systems::playback::Playback {}));
    systems.add_system(Box::new(systems::playback_units::PlaybackUnits {}));
    systems.add_system(Box::new(systems::playback_finished::PlaybackFinished {}));

    systems.add_system(Box::new(systems::team_color_body::TeamColorBody {}));
    systems.add_system(Box::new(systems::health_bar_update::HealthBarUpdate {}));
    systems.add_system(Box::new(systems::display_tank_tracks::DisplayTankTracks {}));
    systems.add_system(Box::new(
        systems::display_capture_flag::DisplayCaptureFlag {},
    ));
    systems.add_system(Box::new(
        systems::display_capture_flag::DisplayCaptureFlag {},
    ));

    // One update cycle to ensure the clock is spawned.
    construct.update();

    Ok(construct)
}

pub fn setup_scenario(
    config: &super::specification::ScenarioConfig,
) -> Result<Construct, Box<dyn std::error::Error>> {
    let mut construct = Construct::new();

    // Add the recorder first, such that on replay its entity id can never collide.
    let recorder_entity = construct.world.add_entity();
    construct
        .world
        .add_component(recorder_entity, components::recording::Recording::new());

    // Add the default systems.
    default::add_components(&mut construct.world);
    default::add_systems(&mut construct.systems);

    if config.recording {
        construct
            .systems
            .add_system(Box::new(systems::record::Record {}));
    }

    match config.pre_setup.as_str() {
        "" => {}
        "playground" => {
            super::playground::populate_dev_world(&mut construct);
        }
        "draw_kinematic_chain" => {
            construct.systems.add_system(Box::new(
                systems::draw_kinematic_chain::DrawKinematicChain {},
            ));
        }
        v => {
            return Err(format!("pre_setup of {v} is not supported").into());
        }
    }

    let world = &mut construct.world;

    // Add teams
    let mut team_set = std::collections::HashMap::<String, specification::Team>::new();
    let mut teams = vec![];
    for team in config.spawn_config.teams.iter() {
        let team_id = components::id_generator::generate_id(world);
        let team_entity = world.add_entity();
        let mut team_component =
            components::team::Team::new(team_id, &team.name, team.color.into());
        team_component.set_comment(team.comment.as_deref());
        if team_set.contains_key(&team.name) {
            // team name occurs twice and the match report won't be able to distinguish, raise
            // an error to avoid indistinguishable results.
            return Err(Box::new(SetupError::new(
                format!("team name {} occurs twice", team.name).as_str(),
            )));
        }
        team_set.insert(team.name.to_owned(), team.clone());
        teams.push(team_component.id());
        world.add_component(team_entity, team_component);
    }

    // Spawn units
    for spawn in config.spawn_config.spawns.iter() {
        let optional_team_component = if let Some(team_index) = spawn.team {
            let team_entity = teams
                .get(team_index)
                .ok_or_else(|| Box::new(SetupError::new("team index out of range")))?;
            Some(components::team_member::TeamMember::new(*team_entity))
        } else {
            None
        };

        fn controller_type_to_control(
            controller_type: &specification::ControllerType,
            control_config: &std::collections::HashMap<String, specification::ControllerType>,
            team_config: &std::collections::HashMap<String, specification::Team>,
        ) -> Result<Box<dyn UnitControl>, Box<dyn std::error::Error>> {
            Ok(match controller_type {
                specification::ControllerType::SwivelShoot => {
                    Box::new(unit_control_builtin::tank_swivel_shoot::TankSwivelShoot::new())
                }
                specification::ControllerType::Idle => {
                    Box::new(unit_control_builtin::idle::Idle {})
                }
                specification::ControllerType::RadioPosition => {
                    Box::new(unit_control_builtin::radio_position::RadioPosition {})
                }
                specification::ControllerType::InterfacePrinter => {
                    Box::new(unit_control_builtin::interface_printer::InterfacePrinter {})
                }
                specification::ControllerType::NaiveShoot => {
                    Box::new(unit_control_builtin::naive_shoot::NaiveShoot::new())
                }
                specification::ControllerType::DiffDriveForwardsBackwards{velocities, duration} => {
                    Box::new(unit_control_builtin::diff_drive_forwards_backwards::DiffDriveForwardsBackwardsControl::new(*velocities, *duration))
                }
                specification::ControllerType::DiffDriveCapturable => {
                    Box::new(unit_control_builtin::diff_drive_capturable::DiffDriveCapturable {})
                }
                #[cfg(not(target_arch = "wasm32"))]
                specification::ControllerType::LibraryLoad { name } => {
                    unit_control_builtin::dynamic_load_control::DynamicLoadControl::new(name)?
                }
                #[cfg(feature = "unit_control_wasm")]
                specification::ControllerType::Wasm(wasmconfig) => {
                    let config = unit_control_wasm::UnitControlWasmConfig {
                        wasm_path: wasmconfig.path.clone().into(),
                        fuel_per_update: wasmconfig.fuel_per_update,
                        reload: wasmconfig.reload,
                        fuel_for_setup: wasmconfig.fuel_for_setup,
                    };
                    Box::new(unit_control_wasm::UnitControlWasm::new_with_config(config)?)
                }
                specification::ControllerType::SequenceControl { controllers } => {
                    let mut v = vec![];
                    for t in controllers.iter() {
                        v.push(controller_type_to_control(t, control_config, team_config)?);
                    }
                    Box::new(unit_control_builtin::sequence_control::SequenceControl::new(v))
                }
                specification::ControllerType::Function ( f ) => {
                    f()
                }
                specification::ControllerType::FromControlConfig{ name } => {
                    let subcontrol = control_config.get(name).ok_or_else(|| {
                        SetupError::new(&format!("requested controller {name} not found"))})?;
                    controller_type_to_control(subcontrol, control_config, team_config)?
                }
                specification::ControllerType::TeamController{ name } => {
                    let subcontrol = team_config.get(name).ok_or_else(|| {
                        SetupError::new(&format!("controlller for requested team {name} not found"))})?;
                    let subcontrol = subcontrol.controller.as_ref().ok_or_else(||{
                        SetupError::new(&format!("team {name} doesn't have a controller but is necessary"))})?;
                    controller_type_to_control(subcontrol, control_config, team_config)?
                }
            })
        }

        let controller: Box<dyn UnitControl> = controller_type_to_control(
            &spawn.controller,
            &config.spawn_config.control_config,
            &team_set,
        )?;
        match spawn.unit {
            specification::Unit::Tank => {
                let unit_config = units::tank::TankSpawnConfig {
                    x: spawn.x,
                    y: spawn.y,
                    yaw: spawn.yaw,
                    controller,
                    team_member: optional_team_component,
                    radio_config: Some(spawn.radio),
                };
                units::tank::spawn_tank(world, unit_config);
            }
            specification::Unit::Artillery => {
                let unit_config = units::artillery::ArtillerySpawnConfig {
                    x: spawn.x,
                    y: spawn.y,
                    yaw: spawn.yaw,
                    controller,
                    team_member: optional_team_component,
                    radio_config: Some(spawn.radio),
                };
                units::artillery::spawn_artillery(world, unit_config);
            }
        }
    }

    let setup_king_of_the_hill = |world: &mut engine::World,
                                  capture_points: &[specification::CapturePoint],
                                  point_limit: Option<f32>|
     -> Result<(), Box<dyn std::error::Error>> {
        for point in capture_points {
            let optional_team_component = if let Some(team_index) = point.team {
                let team_member = teams
                    .get(team_index)
                    .ok_or_else(|| Box::new(SetupError::new("team index out of range")))?;
                Some(team_member)
            } else {
                None
            };
            let config = crate::units::capturable_flag::CapturableFlagConfig {
                x: point.x,
                y: point.y,
                yaw: point.yaw,
                radius: point.radius,
                capture_speed: point.capture_speed,
                initial_owner: optional_team_component.copied(),
                ..Default::default()
            };
            crate::units::capturable_flag::spawn_capturable_flag(world, config);
        }
        // Spawn the king of the hill component.
        let entity = world.add_entity();
        world.add_component(
            entity,
            components::match_king_of_the_hill::MatchKingOfTheHill::new(point_limit),
        );
        Ok(())
    };

    // Setup match.
    match config.match_config.mode.clone() {
        specification::MatchType::None => {}
        specification::MatchType::Domination {
            team_deathmatch_min,
            capture_points,
            point_limit,
        } => {
            let entity = world.add_entity();
            world.add_component(
                entity,
                components::match_domination::MatchDomination::new(team_deathmatch_min),
            );
            let entity = world.add_entity();
            world.add_component(
                entity,
                components::match_team_deathmatch::MatchTeamDeathmatch::new(None),
            );
            setup_king_of_the_hill(world, &capture_points, point_limit)?;
        }
        specification::MatchType::TeamDeathmatch { point_limit } => {
            // Spawn the team deathmatch component.
            let entity = world.add_entity();
            world.add_component(
                entity,
                components::match_team_deathmatch::MatchTeamDeathmatch::new(point_limit),
            );
        }
        specification::MatchType::KingOfTheHill {
            capture_points,
            point_limit,
        } => {
            setup_king_of_the_hill(world, &capture_points, point_limit)?;
        }
    }

    // Configure time limit
    if let Some(time_limit) = config.match_config.time_limit {
        let entity = world.add_entity();
        world.add_component(
            entity,
            components::match_time_limit::MatchTimeLimit::new(time_limit),
        );
    }

    Ok(construct)
}
