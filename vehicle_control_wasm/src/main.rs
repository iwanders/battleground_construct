use battleground_construct::components::vehicle_interface::RegisterInterface;
use vehicle_control_wasm;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut control = vehicle_control_wasm::create_ai();
    let mut interface = RegisterInterface::new();

    interface.add_module(
        "clock",
        3,
        battleground_construct::components::clock::ClockReader::new(),
    );

    control.update(&mut interface);
    Ok(())
}
