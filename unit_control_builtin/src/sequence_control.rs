use crate::UnitControlResult;
use battleground_unit_control::{Interface, UnitControl};

pub struct SequenceControl {
    controllers: Vec<Box<dyn UnitControl>>,
}

impl SequenceControl {
    pub fn new(controllers: Vec<Box<dyn UnitControl>>) -> SequenceControl {
        SequenceControl { controllers }
    }
}

impl UnitControl for SequenceControl {
    fn update(&mut self, interface: &mut dyn Interface) -> UnitControlResult {
        for c in self.controllers.iter_mut() {
            c.update(interface)?;
        }
        Ok(())
    }
}
