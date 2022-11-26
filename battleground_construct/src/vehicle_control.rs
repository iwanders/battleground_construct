/// Plugins will provide a function of this signature.
pub type ControllerSpawn = extern "Rust" fn () -> Box<dyn VehicleAi>;

/// Error type used.
pub type Error = Box<dyn std::error::Error>;

/// Interface to control the vehicle, the ai uses this to interact with the vehicle.
pub trait Interface {
    fn registers(&self) -> usize;
    fn get_u32(&self, register: usize) -> Result<u32, Error>;
    fn set_u32(&mut self, register: usize, value: u32) -> Result<u32, Error>;
}

/// The vehicle ai should implement this trait. Update gets called periodically.
pub trait VehicleAi {
    fn update(&mut self, interface: &mut dyn Interface);
}
