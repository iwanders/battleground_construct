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

/// A register value record.
#[derive(Debug)]
pub enum Value {
    U32(u32),
    F32(f32),
    String(String),
}

/// A register, with a name and a value.
#[derive(Debug)]
pub struct Register {
    name: String,
    value: Value,
}

impl Register {
    /// Retrieve the f32 value from this register.
    pub fn value_f32(&self) -> Option<f32> {
        match self.value {
            Value::F32(v) => Some(v),
            _ => None,
        }
    }
}

impl Register {
    /// Create a new f32 register.
    pub fn new_f32(name: &str, value: f32) -> Self {
        Register {
            name: name.to_owned(),
            value: Value::F32(value),
        }
    }

    /// Create a new u32 register.
    pub fn new_u32(name: &str, value: u32) -> Self {
        Register {
            name: name.to_owned(),
            value: Value::U32(value),
        }
    }
}

/// Type that the vehicle modules will populate and read from.
pub type RegisterMap = std::collections::HashMap<RegisterId, Register>;

pub trait VehicleModule {
    /// Read from the components into the registers.
    fn get_registers(&self, world: &World, registers: &mut RegisterMap);

    /// Set the components' values from the registers.
    fn set_component(&self, world: &mut World, registers: &RegisterMap);
}

pub type ModuleId = u32;
pub type RegisterId = u32;

pub struct Module {
    name: String,
    handler: Box<dyn VehicleModule>,
    registers: std::collections::HashMap<RegisterId, Register>,
}

#[derive(Default)]
pub struct RegisterInterface {
    modules: std::collections::HashMap<ModuleId, Module>,
}

impl RegisterInterface {
    pub fn new() -> Self {
        RegisterInterface::default()
    }

    pub fn add_module_boxed(
        &mut self,
        name: &str,
        index: ModuleId,
        handler: Box<dyn VehicleModule>,
    ) {
        self.modules.insert(
            index,
            Module {
                name: name.to_owned(),
                // index,
                handler,
                registers: Default::default(),
            },
        );
    }

    pub fn add_module<M: VehicleModule>(&mut self, name: &str, index: ModuleId, handler: M)
    where
        M: Sized + 'static,
    {
        self.modules.insert(
            index,
            Module {
                name: name.to_owned(),
                // index,
                handler: Box::new(handler),
                registers: Default::default(),
            },
        );
    }

    pub fn get_registers(&mut self, world: &mut World) {
        for (_id, m) in self.modules.iter_mut() {
            m.handler.get_registers(world, &mut m.registers);
        }
    }
    pub fn set_components(&mut self, world: &mut World) {
        for (_id, m) in self.modules.iter_mut() {
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

    fn get_module_mut(
        &mut self,
        module: ModuleId,
    ) -> Result<&mut Module, battleground_vehicle_control::Error> {
        if let Some(m) = self.modules.get_mut(&module) {
            Ok(m)
        } else {
            Err(InterfaceError::no_such_module(module))
        }
    }

    fn get_register(
        &self,
        module: ModuleId,
        register_index: RegisterId,
    ) -> Result<&Register, battleground_vehicle_control::Error> {
        let m = self.get_module(module)?;
        if let Some(reg) = m.registers.get(&register_index) {
            Ok(reg)
        } else {
            Err(InterfaceError::no_such_register(module, register_index))
        }
    }

    fn get_register_mut(
        &mut self,
        module: ModuleId,
        register_index: RegisterId,
    ) -> Result<&mut Register, battleground_vehicle_control::Error> {
        let m = self.get_module_mut(module)?;
        if let Some(reg) = m.registers.get_mut(&register_index) {
            Ok(reg)
        } else {
            Err(InterfaceError::no_such_register(module, register_index))
        }
    }
}
// This is useless as a component, we need an interior mutability pattern.
// impl Component for RegisterInterface {}

use battleground_vehicle_control::InterfaceError;
impl battleground_vehicle_control::Interface for RegisterInterface {
    fn modules(&self) -> Result<Vec<u32>, battleground_vehicle_control::Error> {
        Ok(self
            .modules
            .iter()
            .map(|(module_index, _module)| *module_index)
            .collect::<_>())
    }

    fn registers(&self, module: u32) -> Result<Vec<u32>, battleground_vehicle_control::Error> {
        let m = self.get_module(module)?;
        Ok(m.registers
            .iter()
            .map(|(register_index, _register)| *register_index)
            .collect::<_>())
    }

    /// Retrieve the name of a particular module.
    fn module_name(&self, module: u32) -> Result<String, battleground_vehicle_control::Error> {
        Ok(self.get_module(module)?.name.clone())
    }

    /// Retrieve a register name
    fn register_name(
        &self,
        module: u32,
        register: u32,
    ) -> Result<String, battleground_vehicle_control::Error> {
        let r = self.get_register(module, register)?;
        Ok(r.name.clone())
    }

    /// Get an f32 register.
    fn get_f32(
        &self,
        module: u32,
        register: u32,
    ) -> Result<f32, battleground_vehicle_control::Error> {
        let r = self.get_register(module, register)?;
        match r.value {
            Value::F32(v) => Ok(v),
            _ => Err(InterfaceError::wrong_type(module, register)),
        }
    }

    /// Set an f32 register.
    fn set_f32(
        &mut self,
        module: u32,
        register: u32,
        value: f32,
    ) -> Result<f32, battleground_vehicle_control::Error> {
        let r = self.get_register_mut(module, register)?;
        match &mut r.value {
            Value::F32(v) => {
                let old = *v;
                *v = value;
                Ok(old)
            }
            _ => Err(InterfaceError::wrong_type(module, register)),
        }
    }
}

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
