/// Plugins will provide a function of this signature.
pub type ControllerSpawn = extern "Rust" fn () -> Box<dyn VehicleControl>;

/// Error type used.
pub type Error = Box<InterfaceError>;

/// Interface to control the vehicle, the ai uses this to interact with the vehicle.
pub trait Interface {

    /// Retrieve the number of available modules.
    fn modules(&self) -> Result<Vec<u32>, Error>;

    /// Retrieve the name of a particular module.
    fn module_name(&self, module: u32) -> Result<String, Error>;

    /// Return the available registers in a particular module.
    fn registers(&self, module: u32) -> Result<Vec<u32>, Error>;

    /// Retrieve a register name
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


#[derive(Clone, Copy, Debug)]
pub enum ErrorType {
    NoSuchModule,
    NoSuchRegister,
    WrongType,
}

#[derive(Debug, Clone)]
pub struct InterfaceError{
    module: u32,
    register: u32,
    error_type: ErrorType,
}
impl InterfaceError {
    pub fn no_such_module(module: u32) -> Box<Self> {
        Box::new(
            InterfaceError{
                module: module,
                register: 0,
                error_type: ErrorType::NoSuchModule,
            }
        )
    }
    pub fn no_such_register(module: u32, register: u32) -> Box<Self> {
        Box::new(
            InterfaceError{
                module: module,
                register: register,
                error_type: ErrorType::NoSuchRegister,
            }
        )
    }
    pub fn wrong_type(module: u32, register: u32) -> Box<Self> {
        Box::new(
            InterfaceError{
                module: module,
                register: register,
                error_type: ErrorType::WrongType,
            }
        )
    }
}

impl std::fmt::Display for InterfaceError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self.error_type {
            ErrorType::NoSuchModule => {write!(f, "{}:# does not exist", self.module)}
            ErrorType::NoSuchRegister => {write!(f, "{}:{} does not exist for this module", self.module, self.register)}
            ErrorType::WrongType => {write!(f, "{}:{} has a different type", self.register, self.module)}
        }
    }
}
