use engine::prelude::*;

/*
let vehicle_interface;
vehicle_interface.add_module::<DiffDriveBase>("base", tank_entity);
vehicle_interface.add_module::<Revolute>("turret", turret_entity);
vehicle_interface.add_module::<Revolute>("barrel", barrel);
vehicle_interface.add_module::<Cannon>("cannon", cannon_entity);
world.add_component(tank_entity, vehicle_control);

// then, in system:
// update vehicle interface.
// run vehicle control
// write vehicle interface back to components.


*/

pub use battleground_vehicle_control::register_interface::{RegisterInterface, Register, RegisterMap, VehicleModule};
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
