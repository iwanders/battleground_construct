use crate::Construct;
// use crate::components;
use super::default;
use super::specification;
use crate::vehicles;

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
                let tank_config = vehicles::tank::TankSpawnConfig {
                    x: spawn.x,
                    y: spawn.y,
                    yaw: spawn.yaw,
                    // controller: Box::new(control::tank_swivel_shoot::TankSwivelShoot {}),
                    controller: Box::new(crate::control::radar_draw::RadarDrawControl {}),
                };
                vehicles::tank::spawn_tank(world, tank_config);
            }
        }
    }

    construct
}
