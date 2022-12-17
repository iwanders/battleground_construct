use battleground_construct::components;
// use battleground_construct::components::vehicle_interface::RegisterInterface;
use battleground_construct::systems;
use battleground_construct::vehicles::tank::{spawn_tank, TankSpawnConfig};
use engine::prelude::*;
// use engine::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut world = World::new();
    let clock_id = world.add_entity();
    world.add_component(clock_id, components::clock::Clock::new());

    let _main_tank = spawn_tank(
        &mut world,
        TankSpawnConfig {
            x: 0.0,
            y: 0.0,
            yaw: 0.0,
            controller: vehicle_control_wasm::create_ai(),
        },
    );

    let mut s = systems::vehicle_control::VehicleControl {};
    s.update(&mut world);

    Ok(())
}
