use crate::Construct;
// use crate::components;
use super::default;
use super::specification;
use crate::units;

pub fn setup_match(config: super::specification::ConstructConfig) -> Construct {
    let mut construct = Construct::new();
    let world = &mut construct.world;
    let systems = &mut construct.systems;
    default::add_components(world);
    default::add_systems(systems);

    for team in config.spawn_config.teams {
        println!("Spawning team: {}", team.name);
    }

    for spawn in config.spawn_config.spawns {
        match spawn.vehicle {
            specification::Vehicle::Tank => {
                let tank_config = units::tank::TankSpawnConfig {
                    x: spawn.x,
                    y: spawn.y,
                    yaw: spawn.yaw,
                    // controller: Box::new(control::tank_swivel_shoot::TankSwivelShoot {}),
                    controller: Box::new(crate::control::radar_draw::RadarDrawControl {}),
                };
                units::tank::spawn_tank(world, tank_config);
            }
        }
    }


    match config.match_config.mode {
        specification::MatchType::None => {},
        specification::MatchType::DeathMatch => {},
        specification::MatchType::KingOfTheHill{capture_points, point_limit} => {
        },

    }

    if let Some(v) = config.match_config.time_limit {
    }

    construct
}
