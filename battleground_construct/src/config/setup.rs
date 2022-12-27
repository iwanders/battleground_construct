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
        let team_id = components::id_generator::generate_id(world);
        let team_entity = world.add_entity();
        let team_component = components::team::Team::new(team_id, &team.name, team.color.into());
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

        let controller: Box<dyn UnitControl> = match spawn.controller {
            specification::ControllerType::SwivelShoot => {
                Box::new(unit_control_builtin::tank_swivel_shoot::TankSwivelShoot {})
            }
            specification::ControllerType::None => Box::new(unit_control_builtin::idle::Idle {}),
            specification::ControllerType::DiffDriveCapturable => {
                Box::new(unit_control_builtin::diff_drive_capturable::DiffDriveCapturable {})
            }
            specification::ControllerType::LibraryLoad { name } => {
                unit_control_builtin::dynamic_load_control::DynamicLoadControl::new(&name)?
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

    use cgmath::Deg;
    let static_cannon = world.add_entity();
    world.add_component(
        static_cannon,
        components::pose::Pose::from_xyz(2.0, 0.0, 15.0).rotated_angle_y(Deg(-90.0)),
    );
    let mut cannon = components::cannon::Cannon::new(components::cannon::CannonConfig {
        fire_effect: std::rc::Rc::new(crate::units::tank::cannon_function),
        reload_time: 1.0,
    });
    cannon.set_firing(true);
    world.add_component(static_cannon, cannon);

    if let Some(time_limit) = config.match_config.time_limit {
        let entity = world.add_entity();
        world.add_component(
            entity,
            components::match_time_limit::MatchTimeLimit::new(time_limit),
        );
    }

    Ok(construct)
}
