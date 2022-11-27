/// Plugins will provide a function of this signature.
pub type ControllerSpawn = fn() -> Box<dyn VehicleControl>;

/// Error type used.
pub type Error = Box<InterfaceError>;

/// Interface to control the vehicle, the ai uses this to interact with the vehicle.
pub trait Interface {
    /// Retrieve the list of module ids that are available.
    fn modules(&self) -> Result<Vec<u32>, Error>;

    /// Retrieve the name of a particular module.
    fn module_name(&self, module: u32) -> Result<String, Error>;

    /// Return the available register ids in a particular module.
    fn registers(&self, module: u32) -> Result<Vec<u32>, Error>;

    /// Retrieve a register name.
    fn register_name(&self, module: u32, register: u32) -> Result<String, Error>;

    /// Get an f32 register.
    fn get_f32(&self, module: u32, register: u32) -> Result<f32, Error>;

    /// Set an f32 register.
    fn set_f32(&mut self, module: u32, register: u32, value: f32) -> Result<f32, Error>;
}

/// The vehicle ai should implement this trait. Update gets called periodically.
pub trait VehicleControl {
    fn update(&mut self, interface: &mut dyn Interface);
}

/// If an error occurs in the interface, the follow boxed error is returned.
#[derive(Debug, Clone)]
pub struct InterfaceError {
    pub module: u32,
    pub register: u32,
    pub error_type: ErrorType,
}

/// The error_type is further specified in the following possible failure modes.
#[derive(Clone, Copy, Debug)]
pub enum ErrorType {
    NoSuchModule,
    NoSuchRegister,
    WrongType,
}

/// Finally, it implements display.
impl std::fmt::Display for InterfaceError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self.error_type {
            ErrorType::NoSuchModule => {
                write!(f, "{}:# does not exist", self.module)
            }
            ErrorType::NoSuchRegister => {
                write!(
                    f,
                    "{}:{} does not exist for this module",
                    self.module, self.register
                )
            }
            ErrorType::WrongType => {
                write!(f, "{}:{} has a different type", self.register, self.module)
            }
        }
    }
}

/// And error, such that it is convertible to Box<dyn std::error:Error>
impl std::error::Error for InterfaceError {}
