use engine::prelude::*;
use crate::components::differential_drive_base::DifferentialDriveBase;

#[derive(Debug)]
pub enum Value {
    U32(u32),
    F32(f32),
    String(String),
}

#[derive(Debug)]
pub struct Register {
    name: String,
    value: Value
}
impl Register {
    pub fn value_f32(&self) -> Option<f32> {
        match self.value { Value::F32(v) => Some(v), _ => None }
    }
}

impl Register {
    pub fn new_f32(name: &str, value: f32) -> Self{
        Register{
            name: name.to_owned(),
            value: Value::F32(value),
        }
    }
    pub fn new_u32(name: &str, value: u32) -> Self{
        Register{
            name: name.to_owned(),
            value: Value::U32(value),
        }
    }
}

pub type RegisterMap = std::collections::HashMap<RegisterId, Register>;

pub trait VehicleModule {
    /// Read from the components into the registers.
    fn get_registers(&self, world: &World, registers: &mut RegisterMap);

    /// Set the components' values from the registers.
    fn set_component(&self, world: &mut World, registers: &RegisterMap);
}

pub struct DifferentialDriveBaseControl{
    entity: EntityId,
}

impl DifferentialDriveBaseControl {
    pub fn new(entity: EntityId) -> Self {
        DifferentialDriveBaseControl{entity}
    }
}

impl VehicleModule for  DifferentialDriveBaseControl {
    fn get_registers(&self, world: &World, registers: &mut RegisterMap)
    {
        registers.clear();
        if let Some(base) = world.component::<DifferentialDriveBase>(self.entity) {
            let vels = base.wheel_velocities();
            registers.insert(0, Register::new_f32("left_wheel_vel", vels.0));
            registers.insert(1, Register::new_f32("right_wheel_vel", vels.1));

            // commanded is the same as reported atm.
            registers.insert(2, Register::new_f32("left_wheel_cmd", vels.0));
            registers.insert(3, Register::new_f32("right_wheel_cmd", vels.1));
        }
    }

    fn set_component(&self, world: &mut World, registers: &RegisterMap)
    {
        if let Some(mut base) = world.component_mut::<DifferentialDriveBase>(self.entity) {
            let left_cmd = registers.get(&2).unwrap().value_f32().expect("Wrong value??");
            let right_cmd = registers.get(&3).unwrap().value_f32().expect("Wrong value??");
            println!("Setting {left_cmd} and {right_cmd}");
            base.set_velocities(left_cmd, right_cmd);
        }
    }

}

pub type ModuleId = u32;
pub type RegisterId = u32;


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

pub struct Module {
    name: String,
    index: ModuleId,
    handler: Box<dyn VehicleModule>,
    registers: std::collections::HashMap<RegisterId, Register>,
}


#[derive(Default)]
pub struct RegisterInterface{
    modules: std::collections::HashMap<ModuleId, Module>,
}

impl RegisterInterface {
    pub fn new() -> Self {
        RegisterInterface::default()
    }

    pub fn add_module(&mut self, name: &str, index: ModuleId, handler: Box<dyn VehicleModule>) {
        self.modules.insert(index, Module{
            name: name.to_owned(),
            index,
            handler,
            registers: Default::default(),
        });
    }

    pub fn get_registers(&mut self, world: &mut World){
        for (id, mut m) in self.modules.iter_mut(){
            m.handler.get_registers(world, &mut m.registers);
        }
    }
    pub fn set_components(&mut self, world: &mut World){
        for (id, m) in self.modules.iter_mut(){
            m.handler.set_component(world, &m.registers);
        }
    }

    fn get_module(&self, module: ModuleId) -> Result<&Module, battleground_vehicle_control::Error> {
        if let Some(m) = self.modules.get(&module) {
            Ok(m)
        } else {
            Err(InterfaceError::no_such_module(module))
        }
    }

    fn get_module_mut(&mut self, module: ModuleId) -> Result<&mut Module, battleground_vehicle_control::Error> {
        if let Some(m) = self.modules.get_mut(&module) {
            Ok(m)
        } else {
            Err(InterfaceError::no_such_module(module))
        }
    }

    fn get_register(&self, module: ModuleId, register_index: RegisterId) -> Result<&Register, battleground_vehicle_control::Error> {
        let m = self.get_module(module)?;
        if let Some(reg) = m.registers.get(&register_index) {
            Ok(reg)
        } else {
            Err(InterfaceError::no_such_register(module, register_index))
        }
    }
    fn get_register_mut(&mut self, module: ModuleId, register_index: RegisterId) -> Result<&mut Register, battleground_vehicle_control::Error> {
        let m = self.get_module_mut(module)?;
        if let Some(reg) = m.registers.get_mut(&register_index) {
            Ok(reg)
        } else {
            Err(InterfaceError::no_such_register(module, register_index))
        }
    }

}
impl Component for RegisterInterface {}


use battleground_vehicle_control::InterfaceError;
impl battleground_vehicle_control::Interface for RegisterInterface {

    fn modules(&self) -> Result<Vec<u32>, battleground_vehicle_control::Error>
    {
        Ok(self.modules.iter().map(|(module_index, _module)|{*module_index}).collect::<_>())
    }

    fn registers(&self, module: u32) -> Result<Vec<u32>, battleground_vehicle_control::Error> {
        let m = self.get_module(module)?;
        Ok(m.registers.iter().map(|(register_index, _register)|{*register_index}).collect::<_>())
    }

    /// Retrieve the name of a particular module.
    fn module_name(&self, module: u32) -> Result<String, battleground_vehicle_control::Error>
    {
        Ok(self.get_module(module)?.name.clone())
    }

    /// Retrieve a register name
    fn register_name(&self, module: u32, register: u32) -> Result<String, battleground_vehicle_control::Error>{
        let r = self.get_register(module, register)?;
        Ok(r.name.clone())
    }

    /// Get an f32 register.
    fn get_f32(&self, module: u32, register: u32) -> Result<f32, battleground_vehicle_control::Error>{
        let r = self.get_register(module, register)?;
        match r.value {
            Value::F32(v) => Ok(v),
            _ => Err(InterfaceError::wrong_type(module, register)),
        }
    }

    /// Set an f32 register.
    fn set_f32(&mut self, module: u32, register: u32, value: f32) -> Result<f32, battleground_vehicle_control::Error>{
        let r = self.get_register_mut(module, register)?;
        match &mut r.value {
            Value::F32(v) => {
                let old = *v;
                *v = value;
                Ok(old)
            },
            _ => Err(InterfaceError::wrong_type(module, register)),
        }
    }
}



// We need interior mutability here because the register things take the entire world as mutable.
use std::rc::Rc;
use std::cell::RefCell;
use std::cell::RefMut;
#[derive(Clone)]
pub struct RegisterInterfaceContainer(Rc<RefCell<RegisterInterface>>);
impl RegisterInterfaceContainer {
    pub fn new(interface: RegisterInterface) -> Self {
        RegisterInterfaceContainer(
            Rc::new(RefCell::new(interface))
        )
    }
    pub fn get_mut(&self) -> RefMut<RegisterInterface>{
        self.0.borrow_mut()
    }
}
impl Component for RegisterInterfaceContainer {}



mod test {
    use super::super::pose::Pose;
    use super::super::revolute::Revolute;
    use super::*;
    // #[test]
    fn nothing() {
    }
}

