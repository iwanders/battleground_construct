use battleground_unit_control::{Interface, UnitControl};
use crate::UnitControlResult;

pub struct Idle {}
impl UnitControl for Idle {
    fn update(&mut self, _interface: &mut dyn Interface) -> UnitControlResult {
        Ok(())
    }
}
