use battleground_vehicle_control::{Interface, VehicleControl};

pub struct Idle {}
impl VehicleControl for Idle {
    fn update(&mut self, _interface: &mut dyn Interface) {}
}
