use battleground_unit_control::{Interface, UnitControl};
pub struct CompositeControl {
    controllers: Vec<Box<dyn UnitControl>>,
}

impl CompositeControl {
    pub fn new(controllers: Vec<Box<dyn UnitControl>>) -> CompositeControl {
        CompositeControl { controllers }
    }
}

impl UnitControl for CompositeControl {
    fn update(&mut self, interface: &mut dyn Interface) {
        for c in self.controllers.iter_mut() {
            c.update(interface);
        }
    }
}
