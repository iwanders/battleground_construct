use engine::prelude::*;

// Most modules look to this file for the UnitModule and Register objects.
pub use battleground_unit_control::register_interface::{
    Register, RegisterInterface, RegisterMap, UnitModule,
};

// We need interior mutability here because the register things take the entire world as mutable.
use std::cell::RefCell;
use std::cell::RefMut;
use std::rc::Rc;

/// Container for the RegisterInterface.
#[derive(Clone)]
pub struct RegisterInterfaceContainer(Rc<RefCell<RegisterInterface>>);
impl RegisterInterfaceContainer {
    pub fn new(interface: RegisterInterface) -> Self {
        RegisterInterfaceContainer(Rc::new(RefCell::new(interface)))
    }
    pub fn get_mut(&self) -> RefMut<RegisterInterface> {
        self.0.borrow_mut()
    }
}
impl Component for RegisterInterfaceContainer {}
