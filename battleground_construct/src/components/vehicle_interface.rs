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
    I32(i32),
    F32(f32),
    Bytes {
        values: Vec<u8>,
        min_len: usize,
        max_len: usize,
    },
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
    /// Retrieve the u32 value from this register.
    pub fn value_i32(&self) -> Option<i32> {
        match self.value {
            Value::I32(v) => Some(v),
            _ => None,
        }
    }

    /// Retrieve the bytes from this register.
    pub fn value_bytes(&self) -> Option<&[u8]> {
        match &self.value {
            Value::Bytes { values, .. } => Some(&values[..]),
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
    pub fn new_i32(name: &str, value: i32) -> Self {
        Register {
            name: name.to_owned(),
            value: Value::I32(value),
        }
    }

    /// Create a new bytes register.
    pub fn new_bytes(name: &str) -> Self {
        Register {
            name: name.to_owned(),
            value: Value::Bytes {
                values: vec![],
                min_len: 0,
                max_len: std::usize::MAX,
            },
        }
    }
}

/// Type that the vehicle modules will populate and read from.
pub type RegisterMap = std::collections::HashMap<RegisterId, Register>;

pub trait VehicleModule {
    /// Read from the components into the registers.
    fn get_registers(&self, _world: &World, _registers: &mut RegisterMap) {}

    /// Set the components' values from the registers.
    fn set_component(&self, _world: &mut World, _registers: &RegisterMap) {}
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

use battleground_vehicle_control::ErrorType;
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
            Err(Self::interface_error(module, 0, ErrorType::NoSuchModule))
        }
    }

    fn get_module_mut(
        &mut self,
        module: ModuleId,
    ) -> Result<&mut Module, battleground_vehicle_control::Error> {
        if let Some(m) = self.modules.get_mut(&module) {
            Ok(m)
        } else {
            Err(Self::interface_error(module, 0, ErrorType::NoSuchModule))
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
            Err(Self::interface_error(
                module,
                register_index,
                ErrorType::NoSuchRegister,
            ))
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
            Err(Self::interface_error(
                module,
                register_index,
                ErrorType::NoSuchRegister,
            ))
        }
    }

    fn interface_error(
        module: u32,
        register: u32,
        error_type: battleground_vehicle_control::ErrorType,
    ) -> Box<InterfaceError> {
        Box::new(InterfaceError {
            module: module,
            register: register,
            error_type: error_type,
        })
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
            _ => Err(RegisterInterface::interface_error(
                module,
                register,
                ErrorType::WrongType,
            )),
        }
    }
    /// Get an u32 register.
    fn get_i32(
        &self,
        module: u32,
        register: u32,
    ) -> Result<i32, battleground_vehicle_control::Error> {
        let r = self.get_register(module, register)?;
        match r.value {
            Value::I32(v) => Ok(v),
            _ => Err(RegisterInterface::interface_error(
                module,
                register,
                ErrorType::WrongType,
            )),
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
            _ => Err(RegisterInterface::interface_error(
                module,
                register,
                ErrorType::WrongType,
            )),
        }
    }

    /// Set an i32 register.
    fn set_i32(
        &mut self,
        module: u32,
        register: u32,
        value: i32,
    ) -> Result<i32, battleground_vehicle_control::Error> {
        let r = self.get_register_mut(module, register)?;
        match &mut r.value {
            Value::I32(v) => {
                let old = *v;
                *v = value;
                Ok(old)
            }
            _ => Err(RegisterInterface::interface_error(
                module,
                register,
                ErrorType::WrongType,
            )),
        }
    }

    /// Get the length required to read a byte register.
    fn get_bytes_len(
        &self,
        module: u32,
        register: u32,
    ) -> Result<usize, battleground_vehicle_control::Error> {
        let r = self.get_register(module, register)?;
        match &r.value {
            Value::Bytes { values, .. } => Ok(values.len()),
            _ => Err(RegisterInterface::interface_error(
                module,
                register,
                ErrorType::WrongType,
            )),
        }
    }

    /// Get the actual bytes of a byte register, returns the number of bytes written.
    fn get_bytes(
        &self,
        module: u32,
        register: u32,
        destination: &mut [u8],
    ) -> Result<usize, battleground_vehicle_control::Error> {
        let r = self.get_register(module, register)?;
        match &r.value {
            Value::Bytes { ref values, .. } => {
                if destination.len() < values.len() {
                    Err(RegisterInterface::interface_error(
                        module,
                        register,
                        ErrorType::ReadOverflow,
                    ))
                } else {
                    // Must be the correct size.
                    destination[0..values.len()].copy_from_slice(&values);
                    Ok(values.len())
                }
            }
            _ => Err(RegisterInterface::interface_error(
                module,
                register,
                ErrorType::WrongType,
            )),
        }
    }

    /// Set a byte register.
    fn set_bytes(
        &mut self,
        module: u32,
        register: u32,
        input_values: &[u8],
    ) -> Result<(), battleground_vehicle_control::Error> {
        let r = self.get_register_mut(module, register)?;
        match &mut r.value {
            Value::Bytes {
                ref mut values,
                min_len,
                max_len,
            } => {
                if values.len() < *min_len {
                    Err(RegisterInterface::interface_error(
                        module,
                        register,
                        ErrorType::WriteUnderflow,
                    ))
                } else if values.len() > *max_len {
                    Err(RegisterInterface::interface_error(
                        module,
                        register,
                        ErrorType::WriteOverflow,
                    ))
                } else {
                    // Must be the correct size.
                    values.clear();
                    values.extend_from_slice(input_values); // do the copy.
                    Ok(())
                }
            }
            _ => Err(RegisterInterface::interface_error(
                module,
                register,
                ErrorType::WrongType,
            )),
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
