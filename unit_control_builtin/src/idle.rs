use battleground_unit_control::{ControlError, Interface, UnitControl};

pub struct Idle {}
impl UnitControl for Idle {
    fn update(&mut self, _interface: &mut dyn Interface) -> Result<(), Box<ControlError>> {
        Ok(())
    }
}
