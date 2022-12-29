use engine::prelude::*;
pub type UnitControlStorage = std::rc::Rc<Box<dyn battleground_unit_control::UnitControl>>;

pub struct UnitController {
    update_interval: f32,
    last_update: f32,
    vehicle_control: UnitControlStorage,
    error: Option<Box<dyn std::error::Error>>,
}

impl UnitController {
    pub fn new(vehicle_control: UnitControlStorage) -> Self {
        UnitController {
            update_interval: 0.01,
            last_update: -0.1,
            vehicle_control,
            error: None,
        }
    }

    pub fn should_update(&self, time: f32) -> bool {
        ((self.last_update + self.update_interval) < time) && self.error.is_none()
    }

    pub fn set_updated(&mut self, time: f32) {
        self.last_update = time;
    }

    pub fn vehicle_control(&mut self) -> &mut dyn battleground_unit_control::UnitControl {
        use std::ops::DerefMut;
        std::rc::Rc::get_mut(&mut self.vehicle_control)
            .expect("Should be exclusive")
            .deref_mut()
    }

    pub fn set_error(&mut self, error: Box<dyn std::error::Error>) {
        self.error = Some(error);
    }
}
impl Component for UnitController {}
