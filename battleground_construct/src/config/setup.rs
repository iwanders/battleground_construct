use super::default;
use super::specification;
use crate::components;
use crate::control;
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

pub fn setup_match(
    config: super::specification::ConstructConfig,
) -> Result<Construct, Box<dyn std::error::Error>> {
    let mut construct = Construct::new();
    let world = &mut construct.world;
    let systems = &mut construct.systems;
    default::add_components(world);
    default::add_systems(systems);

    let mut teams = vec![];
    for team in config.spawn_config.teams {
        let team_entity = world.add_entity();
        teams.push(team_entity);
        let team_component = components::team::Team::new(&team.name, team.color.into());
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

        let controller: Box<dyn UnitControl> = match spawn.controller {
            specification::ControllerType::SwivelShoot => {
                Box::new(control::tank_swivel_shoot::TankSwivelShoot {})
            }
            specification::ControllerType::None => Box::new(control::idle::Idle {}),
            specification::ControllerType::LibraryLoad { name } => {
                control::dynamic_load_control::DynamicLoadControl::new(&name)?
            }
            #[cfg(feature = "unit_control_wasm")]
            specification::ControllerType::Wasm { module } => {
                Box::new(unit_control_wasm::UnitControlWasm::new(&module)?)
            }

            _ => {
                unimplemented!()
            }
        };
        match spawn.vehicle {
            specification::Unit::Tank => {
                let tank_config = units::tank::TankSpawnConfig {
                    x: spawn.x,
                    y: spawn.y,
                    yaw: spawn.yaw,
                    controller,
                    team_member: optional_team_component,
                };
                units::tank::spawn_tank(world, tank_config);
            }
        }
    }

    match config.match_config.mode {
        specification::MatchType::None => {}
        specification::MatchType::DeathMatch => {}
        specification::MatchType::KingOfTheHill {
            capture_points: _,
            point_limit: _,
        } => {}
    }

    if let Some(_v) = config.match_config.time_limit {}

    Ok(construct)
}
