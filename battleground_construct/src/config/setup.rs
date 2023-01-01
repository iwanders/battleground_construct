use super::default;
use super::specification;
use crate::components;
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

pub fn setup_scenario(
    config: super::specification::ScenarioConfig,
) -> Result<Construct, Box<dyn std::error::Error>> {
    let mut construct = Construct::new();
    let world = &mut construct.world;
    let systems = &mut construct.systems;
    default::add_components(world);
    default::add_systems(systems);

    let mut team_set = std::collections::HashSet::<String>::new();
    let mut teams = vec![];
    for team in config.spawn_config.teams {
        let team_id = components::id_generator::generate_id(world);
        let team_entity = world.add_entity();
        let team_component = components::team::Team::new(team_id, &team.name, team.color.into());
        if team_set.contains(&team.name) {
            // team name occurs twice and the match report won't be able to distinguish, raise
            // an error to avoid indistinguishable results.
            return Err(Box::new(SetupError::new(
                format!("team name {} occurs twice", team.name).as_str(),
            )));
        }
        team_set.insert(team.name.to_owned());
        teams.push(team_component.id());
        world.add_component(team_entity, team_component);
    }

    for spawn in config.spawn_config.spawns {
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
        ) -> Result<Box<dyn UnitControl>, Box<dyn std::error::Error>> {
            Ok(match controller_type {
                specification::ControllerType::SwivelShoot => {
                    Box::new(unit_control_builtin::tank_swivel_shoot::TankSwivelShoot::new())
                }
                specification::ControllerType::None => {
                    Box::new(unit_control_builtin::idle::Idle {})
                }
                specification::ControllerType::RadioPosition => {
                    Box::new(unit_control_builtin::radio_position::RadioPosition {})
                }
                specification::ControllerType::InterfacePrinter => {
                    Box::new(unit_control_builtin::interface_printer::InterfacePrinter {})
                }
                specification::ControllerType::TankNaiveShoot => {
                    Box::new(unit_control_builtin::tank_naive_shoot::TankNaiveShoot::new())
                }
                specification::ControllerType::DiffDriveForwardsBackwards{velocities, duration} => {
                    Box::new(unit_control_builtin::diff_drive_forwards_backwards::DiffDriveForwardsBackwardsControl::new(*velocities, *duration))
                }
                specification::ControllerType::DiffDriveCapturable => {
                    Box::new(unit_control_builtin::diff_drive_capturable::DiffDriveCapturable {})
                }
                specification::ControllerType::LibraryLoad { name } => {
                    unit_control_builtin::dynamic_load_control::DynamicLoadControl::new(&name)?
                }
                #[cfg(feature = "unit_control_wasm")]
                specification::ControllerType::Wasm(wasmconfig) => {
                    let config = unit_control_wasm::UnitControlWasmConfig {
                        wasm_path: wasmconfig.path.clone().into(),
                        fuel_per_update: wasmconfig.fuel_per_update,
                        print_exports: wasmconfig.print_exports,
                        fuel_for_setup: wasmconfig.fuel_for_setup,
                    };
                    Box::new(unit_control_wasm::UnitControlWasm::new_with_config(config)?)
                }
                specification::ControllerType::SequenceControl { controllers } => {
                    let mut v = vec![];
                    for t in controllers.iter() {
                        v.push(controller_type_to_control(t, control_config)?);
                    }
                    Box::new(unit_control_builtin::sequence_control::SequenceControl::new(v))
                }
                specification::ControllerType::Function ( f ) => {
                    f()
                }
                specification::ControllerType::FromControlConfig{ name } => {
                    let subcontrol = control_config.get(name).ok_or_else(|| {
                        SetupError::new(&format!("requested controller {} not found", name))})?;
                    controller_type_to_control(subcontrol, control_config)?
                }
            })
        }

        let controller: Box<dyn UnitControl> =
            controller_type_to_control(&spawn.controller, &config.spawn_config.control_config)?;
        match spawn.vehicle {
            specification::Unit::Tank => {
                let tank_config = units::tank::TankSpawnConfig {
                    x: spawn.x,
                    y: spawn.y,
                    yaw: spawn.yaw,
                    controller,
                    team_member: optional_team_component,
                    radio_config: Some(spawn.radio),
                };
                units::tank::spawn_tank(world, tank_config);
            }
        }
    }

    match config.match_config.mode {
        specification::MatchType::None => {}
        specification::MatchType::DeathMatch => {}
        specification::MatchType::KingOfTheHill {
            capture_points,
            point_limit,
        } => {
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
        }
    }

    if let Some(time_limit) = config.match_config.time_limit {
        let entity = world.add_entity();
        world.add_component(
            entity,
            components::match_time_limit::MatchTimeLimit::new(time_limit),
        );
    }

    Ok(construct)
}
