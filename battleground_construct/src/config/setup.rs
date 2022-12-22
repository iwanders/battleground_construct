use crate::Construct;
// use crate::components;
use super::default;
use super::specification;
use crate::control;
use crate::units;
use battleground_vehicle_control::VehicleControl;

pub fn setup_match(
    config: super::specification::ConstructConfig,
) -> Result<Construct, Box<dyn std::error::Error>> {
    let mut construct = Construct::new();
    let world = &mut construct.world;
    let systems = &mut construct.systems;
    default::add_components(world);
    default::add_systems(systems);

    for team in config.spawn_config.teams {
        println!("Spawning team: {}", team.name);
    }

    for spawn in config.spawn_config.spawns {
        let controller: Box<dyn VehicleControl> = match spawn.controller {
            specification::ControllerType::SwivelShoot => {
                Box::new(control::tank_swivel_shoot::TankSwivelShoot {})
            }
            specification::ControllerType::None => Box::new(control::idle::Idle {}),
            specification::ControllerType::LibraryLoad { name } => {
                control::dynamic_load_control::DynamicLoadControl::new(&name)?
            }
            #[cfg(feature = "vehicle_control_wasm")]
            specification::ControllerType::Wasm { module } => {
                Box::new(vehicle_control_wasm::VehicleControlWasm::new(&module)?)
            }

            _ => {
                unimplemented!()
            }
        };
        match spawn.vehicle {
            specification::Vehicle::Tank => {
                let tank_config = units::tank::TankSpawnConfig {
                    x: spawn.x,
                    y: spawn.y,
                    yaw: spawn.yaw,
                    controller,
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
