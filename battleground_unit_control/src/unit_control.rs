use crate::interface::Interface;

/// The unit controller should implement this trait.
pub trait UnitControl {
    /// Function used to control the unit.
    fn update(&mut self, interface: &mut dyn Interface) -> Result<(), Box<dyn std::error::Error>>;
}
