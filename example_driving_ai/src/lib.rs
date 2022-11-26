use battleground_construct::vehicle_control::{ControllerSpawn, Error, Interface, VehicleAi};

struct SimpleAi {}

impl SimpleAi {
    fn new() -> Self {
        SimpleAi {}
    }
}

impl VehicleAi for SimpleAi {
    fn update(&mut self, interface: &mut dyn Interface) {
    println!("We got called");
    }
}

#[no_mangle]
pub extern "Rust" fn create_ai() -> Box<dyn VehicleAi> {
    Box::new(SimpleAi::new())
}

