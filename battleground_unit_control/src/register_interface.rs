use engine::World;

type BoxedError = Box<crate::interface::InterfaceError>;
use crate::interface::InterfaceErrorType;

/// A register value record.
#[derive(Debug)]
pub enum Value {
    /// An i32 value.
    I32(i32),
    /// An f32 value.
    F32(f32),
    /// A bytes register, with min and max length.
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
    /// Retrieve the i32 value from this register.
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
    /// Retrieve writable bytes.
    pub fn value_bytes_mut(&mut self) -> Option<&mut Vec<u8>> {
        match &mut self.value {
            Value::Bytes { ref mut values, .. } => Some(values),
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
                max_len: usize::MAX,
            },
        }
    }

    /// Create a new bytes register with a limit.
    pub fn new_bytes_max(name: &str, max_len: usize) -> Self {
        Register {
            name: name.to_owned(),
            value: Value::Bytes {
                values: vec![],
                min_len: 0,
                max_len,
            },
        }
    }
}

/// Type that the unit modules will populate and read from.
pub type RegisterMap = std::collections::HashMap<RegisterId, Register>;

/// Trait that should be implemented by modules exposed through an interface.
pub trait UnitModule {
    /// Read from the components into the registers.
    fn get_registers(&self, _world: &World, _registers: &mut RegisterMap) {}

    /// Set the components' values from the registers.
    fn set_component(&self, _world: &mut World, _registers: &RegisterMap) {}
}

/// Module id type
pub type ModuleId = u32;
/// Register id type
pub type RegisterId = u32;

/// A module representing a single module
pub struct Module {
    /// The name of this module.
    name: String,

    /// The handler for this module, to get the registers and set the component states.
    handler: Option<Box<dyn UnitModule>>,

    /// The registers themselves.
    registers: std::collections::HashMap<RegisterId, Register>,
}

impl Module {
    pub fn read_interface(
        &mut self,
        module: ModuleId,
        interface: &dyn crate::Interface,
    ) -> Result<(), BoxedError> {
        for reg_id in interface.registers(module)? {
            let name = interface.register_name(module, reg_id)?;
            let reg = match interface.register_type(module, reg_id)? {
                crate::RegisterType::I32 => {
                    Register::new_i32(&name, interface.get_i32(module, reg_id)?)
                }
                crate::RegisterType::F32 => {
                    Register::new_f32(&name, interface.get_f32(module, reg_id)?)
                }
                crate::RegisterType::Bytes => {
                    let mut reg = Register::new_bytes(&name);
                    let b = reg.value_bytes_mut().unwrap();
                    b.clear();
                    b.resize(interface.get_bytes_len(module, reg_id)?, 0u8);
                    interface.get_bytes(module, reg_id, &mut b[..])?;
                    reg
                }
            };
            self.add_register(reg_id, reg);
        }
        Ok(())
    }

    pub fn add_register(&mut self, register_id: RegisterId, value: Register) {
        self.registers.insert(register_id, value);
    }

    pub fn remove_register(&mut self, register_id: RegisterId) {
        self.registers.remove(&register_id);
    }

    pub fn write_interface(
        &self,
        module: ModuleId,
        interface: &mut dyn crate::Interface,
    ) -> Result<(), BoxedError> {
        for (k, v) in self.registers.iter() {
            let register = *k;
            match v.value {
                Value::I32(v) => {
                    interface.set_i32(module, register, v)?;
                }
                Value::F32(v) => {
                    interface.set_f32(module, register, v)?;
                }
                Value::Bytes { ref values, .. } => {
                    interface.set_bytes(module, register, &values[..])?;
                }
            }
        }
        Ok(())
    }
}

/// The register interface holds a bunch of modules, it is used by the unit interface and the
/// wasm_interface.
#[derive(Default)]
pub struct RegisterInterface {
    modules: std::collections::HashMap<ModuleId, Module>,
}

impl RegisterInterface {
    pub fn new() -> Self {
        RegisterInterface::default()
    }

    /// Copy all registers from the interface to self.
    pub fn read_interface(&mut self, interface: &dyn crate::Interface) -> Result<(), BoxedError> {
        self.modules.clear();
        for module in interface.modules()? {
            let name = interface.module_name(module)?;
            self.modules.insert(
                module,
                Module {
                    name,
                    handler: None,
                    registers: Default::default(),
                },
            );
            self.modules
                .get_mut(&module)
                .unwrap()
                .read_interface(module, interface)?;
        }
        Ok(())
    }

    /// Copy all registers from self to the interface.
    pub fn write_interface(&self, interface: &mut dyn crate::Interface) -> Result<(), BoxedError> {
        for module in interface.modules()? {
            self.modules
                .get(&module)
                .unwrap()
                .write_interface(module, interface)?;
        }
        Ok(())
    }

    pub fn add_module_boxed(&mut self, name: &str, index: ModuleId, handler: Box<dyn UnitModule>) {
        self.modules.insert(
            index,
            Module {
                name: name.to_owned(),
                // index,
                handler: Some(handler),
                registers: Default::default(),
            },
        );
    }

    pub fn add_module<M: UnitModule + Sized + 'static>(&mut self, name: &str, index: ModuleId, handler: M)
    {
        self.modules.insert(
            index,
            Module {
                name: name.to_owned(),
                // index,
                handler: Some(Box::new(handler)),
                registers: Default::default(),
            },
        );
    }

    pub fn get_registers(&mut self, world: &mut World) {
        for (_id, m) in self.modules.iter_mut() {
            if let Some(ref handler) = m.handler {
                handler.get_registers(world, &mut m.registers);
            }
        }
    }
    pub fn set_components(&mut self, world: &mut World) {
        for (_id, m) in self.modules.iter_mut() {
            if let Some(ref handler) = m.handler {
                handler.set_component(world, &m.registers);
            }
        }
    }

    fn get_module(&self, module: ModuleId) -> Result<&Module, BoxedError> {
        if let Some(m) = self.modules.get(&module) {
            Ok(m)
        } else {
            Err(Self::interface_error(
                module,
                0,
                InterfaceErrorType::NoSuchModule,
            ))
        }
    }

    pub fn get_module_mut(&mut self, module: ModuleId) -> Result<&mut Module, BoxedError> {
        if let Some(m) = self.modules.get_mut(&module) {
            Ok(m)
        } else {
            Err(Self::interface_error(
                module,
                0,
                InterfaceErrorType::NoSuchModule,
            ))
        }
    }

    fn get_register(
        &self,
        module: ModuleId,
        register_index: RegisterId,
    ) -> Result<&Register, BoxedError> {
        let m = self.get_module(module)?;
        if let Some(reg) = m.registers.get(&register_index) {
            Ok(reg)
        } else {
            Err(Self::interface_error(
                module,
                register_index,
                InterfaceErrorType::NoSuchRegister,
            ))
        }
    }

    fn get_register_mut(
        &mut self,
        module: ModuleId,
        register_index: RegisterId,
    ) -> Result<&mut Register, BoxedError> {
        let m = self.get_module_mut(module)?;
        if let Some(reg) = m.registers.get_mut(&register_index) {
            Ok(reg)
        } else {
            Err(Self::interface_error(
                module,
                register_index,
                InterfaceErrorType::NoSuchRegister,
            ))
        }
    }

    fn interface_error(module: u32, register: u32, error_type: InterfaceErrorType) -> BoxedError {
        Box::new(InterfaceError {
            module,
            register,
            error_type,
        })
    }
}

/// Finally, implement the interface trait for the register interface.
use crate::interface::InterfaceError;
impl crate::interface::Interface for RegisterInterface {
    fn modules(&self) -> Result<Vec<u32>, BoxedError> {
        let mut modules: Vec<u32> = self.modules.keys().copied().collect();
        modules.sort();
        Ok(modules)
    }

    fn registers(&self, module: u32) -> Result<Vec<u32>, BoxedError> {
        let m = self.get_module(module)?;
        let mut regs: Vec<u32> = m.registers.keys().copied().collect();
        regs.sort();
        Ok(regs)
    }

    /// Retrieve the name of a particular module.
    fn module_name(&self, module: u32) -> Result<String, BoxedError> {
        Ok(self.get_module(module)?.name.clone())
    }

    /// Retrieve a register name
    fn register_name(&self, module: u32, register: u32) -> Result<String, BoxedError> {
        let r = self.get_register(module, register)?;
        Ok(r.name.clone())
    }

    /// Retrieve a register type.
    fn register_type(&self, module: u32, register: u32) -> Result<crate::RegisterType, BoxedError> {
        let r = self.get_register(module, register)?;
        Ok(match &r.value {
            Value::I32(..) => crate::RegisterType::I32,
            Value::F32(..) => crate::RegisterType::F32,
            Value::Bytes { .. } => crate::RegisterType::Bytes,
        })
    }

    /// Get an f32 register.
    fn get_f32(&self, module: u32, register: u32) -> Result<f32, BoxedError> {
        let r = self.get_register(module, register)?;
        match r.value {
            Value::F32(v) => Ok(v),
            _ => Err(RegisterInterface::interface_error(
                module,
                register,
                InterfaceErrorType::WrongType,
            )),
        }
    }
    /// Get an u32 register.
    fn get_i32(&self, module: u32, register: u32) -> Result<i32, BoxedError> {
        let r = self.get_register(module, register)?;
        match r.value {
            Value::I32(v) => Ok(v),
            _ => Err(RegisterInterface::interface_error(
                module,
                register,
                InterfaceErrorType::WrongType,
            )),
        }
    }

    /// Set an f32 register.
    fn set_f32(&mut self, module: u32, register: u32, value: f32) -> Result<f32, BoxedError> {
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
                InterfaceErrorType::WrongType,
            )),
        }
    }

    /// Set an i32 register.
    fn set_i32(&mut self, module: u32, register: u32, value: i32) -> Result<i32, BoxedError> {
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
                InterfaceErrorType::WrongType,
            )),
        }
    }

    /// Get the length required to read a byte register.
    fn get_bytes_len(&self, module: u32, register: u32) -> Result<usize, BoxedError> {
        let r = self.get_register(module, register)?;
        match &r.value {
            Value::Bytes { values, .. } => Ok(values.len()),
            _ => Err(RegisterInterface::interface_error(
                module,
                register,
                InterfaceErrorType::WrongType,
            )),
        }
    }

    /// Get the actual bytes of a byte register, returns the number of bytes written.
    fn get_bytes(
        &self,
        module: u32,
        register: u32,
        destination: &mut [u8],
    ) -> Result<usize, BoxedError> {
        let r = self.get_register(module, register)?;
        match &r.value {
            Value::Bytes { ref values, .. } => {
                if destination.len() < values.len() {
                    Err(RegisterInterface::interface_error(
                        module,
                        register,
                        InterfaceErrorType::ReadOverflow,
                    ))
                } else {
                    // Must be the correct size.
                    destination[0..values.len()].copy_from_slice(values);
                    Ok(values.len())
                }
            }
            _ => Err(RegisterInterface::interface_error(
                module,
                register,
                InterfaceErrorType::WrongType,
            )),
        }
    }

    /// Set a byte register.
    fn set_bytes(
        &mut self,
        module: u32,
        register: u32,
        input_values: &[u8],
    ) -> Result<(), BoxedError> {
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
                        InterfaceErrorType::WriteUnderflow,
                    ))
                } else if values.len() > *max_len {
                    Err(RegisterInterface::interface_error(
                        module,
                        register,
                        InterfaceErrorType::WriteOverflow,
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
                InterfaceErrorType::WrongType,
            )),
        }
    }
}
