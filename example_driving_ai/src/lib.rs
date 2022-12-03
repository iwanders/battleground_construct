use battleground_vehicle_control::{Interface, VehicleAi};

struct SimpleAi {}

impl SimpleAi {
    fn new() -> Self {
        SimpleAi {}
    }
}

impl VehicleAi for SimpleAi {
    fn update(&mut self, _interface: &mut dyn Interface) {
        println!("We got called");
    }
}

#[no_mangle]
pub fn create_ai() -> Box<dyn VehicleAi> {
    Box::new(SimpleAi::new())
}
