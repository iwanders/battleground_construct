use battleground_vehicle_control::{Interface, VehicleControl};

struct SimpleAi {}

impl SimpleAi {
    fn new() -> Self {
        SimpleAi {}
    }
}

impl VehicleControl for SimpleAi {
    fn update(&mut self, _interface: &mut dyn Interface) {
        println!("We got called");
    }
}

#[no_mangle]
pub fn create_ai() -> Box<dyn VehicleControl> {
    Box::new(SimpleAi::new())
}
