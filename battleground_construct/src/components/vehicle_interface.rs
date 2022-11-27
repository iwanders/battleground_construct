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
    id: RegisterId,
    name: String,
    value: Value
}
impl Register {
    pub fn value_f32(&self) -> Option<f32> {
        match self.value { Value::F32(v) => Some(v), _ => None }
    }
}

impl Register {
    pub fn new_f32(id: RegisterId, name: &str, value: f32) -> Self{
        Register{
            id,
            name: name.to_owned(),
            value: Value::F32(value),
        }
    }
    pub fn new_u32(id: RegisterId, name: &str, value: u32) -> Self{
        Register{
            id,
            name: name.to_owned(),
            value: Value::U32(value),
        }
    }
}

pub trait VehicleModule {
    /// Read from the components into the registers.
    fn get_registers(&self, world: &World) -> Vec<Register>;

    /// Set the components' values from the registers.
    fn set_component(&self, world: &mut World, registers: &[Register]);
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
    fn get_registers(&self, world: &World) -> Vec<Register>
    {
        vec![
            Register::new_f32(0, "left_wheel_vel", 0.0),
            Register::new_f32(1, "right_wheel_vel", 0.0),
            Register::new_f32(2, "left_wheel_command", 0.0),
            Register::new_f32(3, "right_wheel_command", 0.0),
            ]
    }

    fn set_component(&self, world: &mut World, registers: &[Register])
    {
        if let Some(mut base) = world.component_mut::<DifferentialDriveBase>(self.entity) {
            let left_cmd = registers[2].value_f32().expect("Wrong value??");
            let right_cmd = registers[0].value_f32().expect("Wrong value??");
            base.set_velocities(left_cmd, right_cmd);
        }
    }

}

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

struct Module {
    module_name: String,
    module_offset: RegisterId,
    handler: Box<dyn VehicleModule>,
    registers: Vec<Register>,
}


#[derive(Default)]
pub struct RegisterInterface{
    modules: Vec<Module>,
}

impl RegisterInterface {
    pub fn new() -> Self {
        RegisterInterface::default()
    }

    pub fn add_module(&mut self, module_name: &str, module_offset: RegisterId, handler: Box<dyn VehicleModule>) {
        self.modules.push(Module{
            module_name: module_name.to_owned(),
            module_offset,
            handler,
            registers: vec![],
        })
    }

    pub fn get_registers(&mut self, world: &mut World){
        for m in self.modules.iter_mut(){
            m.registers = m.handler.get_registers(world);
        }
    }
    pub fn set_components(&mut self, world: &mut World){
        for m in self.modules.iter_mut(){
            m.handler.set_component(world, &m.registers);
        }
    }
}
impl Component for RegisterInterface {}


impl battleground_vehicle_control::Interface for RegisterInterface {
    fn registers(&self) -> usize {
        // todo!()
        self.modules.iter().map(|v|{v.registers.len()}).sum()
    }
    fn get_u32(&self, _: usize) -> Result<u32, Box<(dyn std::error::Error + 'static)>> {
        // todo!()
        Ok(0)
    }
    fn set_u32(&mut self, _: usize, _: u32) -> Result<u32, Box<(dyn std::error::Error + 'static)>> {
        // todo!()
        Ok(0)
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

