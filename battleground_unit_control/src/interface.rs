/// Error type used.
type BoxedError = Box<InterfaceError>;

/// Enum to denote register type.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum RegisterType {
    /// Register will hold an i32.
    I32,
    /// Register will hold an f32.
    F32,
    /// Register will hold bytes.
    Bytes,
}

impl TryFrom<u32> for RegisterType {
    type Error = ();
    fn try_from(v: u32) -> Result<Self, Self::Error> {
        match v {
            x if x == RegisterType::I32 as u32 => Ok(RegisterType::I32),
            x if x == RegisterType::F32 as u32 => Ok(RegisterType::F32),
            x if x == RegisterType::Bytes as u32 => Ok(RegisterType::Bytes),
            _ => Err(()),
        }
    }
}

/// Interface to control the unit, the unit controller uses this to interact with the unit.
pub trait Interface {
    /// Retrieve the list of module ids that are available.
    fn modules(&self) -> Result<Vec<u32>, BoxedError>;

    /// Retrieve the name of a particular module.
    fn module_name(&self, module: u32) -> Result<String, BoxedError>;

    /// Return the available register ids in a particular module.
    fn registers(&self, module: u32) -> Result<Vec<u32>, BoxedError>;

    /// Retrieve a register name.
    fn register_name(&self, module: u32, register: u32) -> Result<String, BoxedError>;

    /// Retrieve a register type.
    fn register_type(&self, module: u32, register: u32) -> Result<RegisterType, BoxedError>;

    /// Get an i32 register.
    fn get_i32(&self, module: u32, register: u32) -> Result<i32, BoxedError>;

    /// Set an i32 register.
    fn set_i32(&mut self, module: u32, register: u32, value: i32) -> Result<i32, BoxedError>;

    /// Get an f32 register.
    fn get_f32(&self, module: u32, register: u32) -> Result<f32, BoxedError>;

    /// Set an f32 register.
    fn set_f32(&mut self, module: u32, register: u32, value: f32) -> Result<f32, BoxedError>;

    /// Get the length required to read a byte register.
    fn get_bytes_len(&self, module: u32, register: u32) -> Result<usize, BoxedError>;

    /// Get the actual bytes of a byte register, returns the number of bytes written.
    fn get_bytes(
        &self,
        module: u32,
        register: u32,
        destination: &mut [u8],
    ) -> Result<usize, BoxedError>;

    /// Set a byte register.
    fn set_bytes(&mut self, module: u32, register: u32, values: &[u8]) -> Result<(), BoxedError>;
}

/// If an error occurs in the interface, the following boxed error is returned.
#[derive(Debug, Clone)]
pub struct InterfaceError {
    /// The module interacted with when this error occured.
    pub module: u32,
    /// The register interacted with when this error occured.
    pub register: u32,
    /// The type of error that occured.
    pub error_type: InterfaceErrorType,
}

/// The error_type is further specified in the following possible failure modes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InterfaceErrorType {
    /// This module does not exist.
    NoSuchModule,
    /// This register does not exist in this module.
    NoSuchRegister,
    /// This register is of another type.
    WrongType,
    /// The destination buffer could not hold enough values.
    ReadOverflow,
    /// Too many values are being written, the register takes less.
    WriteOverflow,
    /// Too few values are being written, the register requires more.
    WriteUnderflow,
}

impl TryFrom<u32> for InterfaceErrorType {
    type Error = ();
    fn try_from(v: u32) -> Result<Self, Self::Error> {
        match v {
            x if x == InterfaceErrorType::NoSuchModule as u32 => {
                Ok(InterfaceErrorType::NoSuchModule)
            }
            x if x == InterfaceErrorType::NoSuchRegister as u32 => {
                Ok(InterfaceErrorType::NoSuchRegister)
            }
            x if x == InterfaceErrorType::WrongType as u32 => Ok(InterfaceErrorType::WrongType),
            x if x == InterfaceErrorType::ReadOverflow as u32 => {
                Ok(InterfaceErrorType::ReadOverflow)
            }
            x if x == InterfaceErrorType::WriteOverflow as u32 => {
                Ok(InterfaceErrorType::WriteOverflow)
            }
            x if x == InterfaceErrorType::WriteUnderflow as u32 => {
                Ok(InterfaceErrorType::WriteUnderflow)
            }
            _ => Err(()),
        }
    }
}

/// Finally, it implements display.
impl std::fmt::Display for InterfaceError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self.error_type {
            InterfaceErrorType::NoSuchModule => {
                write!(f, "{:0>8x}:# does not exist", self.module)
            }
            InterfaceErrorType::NoSuchRegister => {
                write!(
                    f,
                    "{:0>8x}:{:0>8x} does not exist for this module",
                    self.module, self.register
                )
            }
            InterfaceErrorType::WrongType => {
                write!(
                    f,
                    "{:0>8x}:{:0>8x} has a different type",
                    self.register, self.module
                )
            }
            InterfaceErrorType::ReadOverflow => {
                write!(
                    f,
                    "{:0>8x}:{:0>8x} destination buffer not large enough",
                    self.register, self.module
                )
            }
            InterfaceErrorType::WriteOverflow => {
                write!(
                    f,
                    "{:0>8x}:{:0>8x} input data exceeds register size",
                    self.register, self.module
                )
            }
            InterfaceErrorType::WriteUnderflow => {
                write!(
                    f,
                    "{:0>8x}:{:0>8x} register size exceeds input data",
                    self.register, self.module
                )
            }
        }
    }
}

/// And error, such that it is convertible to [`Box<dyn std::error:Error>`]
impl std::error::Error for InterfaceError {}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_error_conversion() {
        assert_eq!(
            InterfaceErrorType::NoSuchModule,
            (InterfaceErrorType::NoSuchModule as u32)
                .try_into()
                .unwrap()
        );
        assert_eq!(
            InterfaceErrorType::NoSuchRegister,
            (InterfaceErrorType::NoSuchRegister as u32)
                .try_into()
                .unwrap()
        );
        assert_eq!(
            InterfaceErrorType::WrongType,
            (InterfaceErrorType::WrongType as u32).try_into().unwrap()
        );
        assert_eq!(
            InterfaceErrorType::ReadOverflow,
            (InterfaceErrorType::ReadOverflow as u32)
                .try_into()
                .unwrap()
        );
        assert_eq!(
            InterfaceErrorType::WriteOverflow,
            (InterfaceErrorType::WriteOverflow as u32)
                .try_into()
                .unwrap()
        );
        assert_eq!(
            InterfaceErrorType::WriteUnderflow,
            (InterfaceErrorType::WriteUnderflow as u32)
                .try_into()
                .unwrap()
        );
    }
    #[test]
    fn test_register_type_conversion() {
        assert_eq!(
            RegisterType::I32,
            (RegisterType::I32 as u32).try_into().unwrap()
        );
        assert_eq!(
            RegisterType::F32,
            (RegisterType::F32 as u32).try_into().unwrap()
        );
        assert_eq!(
            RegisterType::Bytes,
            (RegisterType::Bytes as u32).try_into().unwrap()
        );
    }
}
