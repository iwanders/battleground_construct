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
            // don't run on first cycle, such that we see the mesh if panic on first.
            last_update: 0.0,
            vehicle_control,
            error: None,
        }
    }

    pub fn update_interval(&self) -> f32 {
        self.update_interval
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

    pub fn error(&self) -> Option<&dyn std::error::Error> {
        if let Some(ref e) = self.error {
            return Some(&**e);
        }
        None
    }
}
impl Component for UnitController {}

use crate::components::unit_interface::{Register, RegisterMap, UnitModule};

#[derive(Debug, Clone, Copy)]
pub struct UnitControllerModule {
    entity: EntityId,
}

impl UnitControllerModule {
    pub fn new(entity: EntityId) -> Self {
        UnitControllerModule { entity }
    }
}
impl Component for UnitControllerModule {}

impl UnitModule for UnitControllerModule {
    fn get_registers(&self, world: &World, registers: &mut RegisterMap) {
        let value = world
            .component::<UnitController>(self.entity)
            .expect("entity should have controller")
            .update_interval();
        registers.insert(
            battleground_unit_control::modules::controller::REG_CONTROLLER_UPDATE_INTERVAL,
            Register::new_f32("update_interval", value),
        );
    }
}
