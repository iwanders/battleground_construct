use engine::prelude::*;

pub type VehicleControlStorage = std::rc::Rc<Box<dyn battleground_vehicle_control::VehicleControl>>;

#[derive(Clone)]
pub struct VehicleController {
    update_interval: f32,
    last_update: f32,
    vehicle_control: VehicleControlStorage,
}

impl VehicleController {
    pub fn new(vehicle_control: VehicleControlStorage) -> Self {
        VehicleController {
            update_interval: 0.1,
            last_update: -0.1,
            vehicle_control,
        }
    }

    pub fn should_update(
        &mut self,
        time: f32,
    ) -> Option<&mut dyn battleground_vehicle_control::VehicleControl> {
        if (self.last_update + self.update_interval) < time {
            self.last_update = time;
            return Some(self.vehicle_control());
        }
        None
    }

    fn vehicle_control(&mut self) -> &mut dyn battleground_vehicle_control::VehicleControl {
        use std::ops::DerefMut;
        std::rc::Rc::get_mut(&mut self.vehicle_control)
            .expect("Should be exclusive")
            .deref_mut()
    }
}
impl Component for VehicleController {}

pub struct DummyVehicleControl {}
impl battleground_vehicle_control::VehicleControl for DummyVehicleControl {
    fn update(&mut self, _interface: &mut dyn battleground_vehicle_control::Interface) {
        println!("Update ran.");
    }
}
