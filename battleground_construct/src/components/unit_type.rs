use engine::prelude::*;

pub use battleground_unit_control::units::UnitType as UnitTypeEnum;

#[derive(Debug, Clone, Copy)]
pub struct UnitType {
    unit_type: UnitTypeEnum,
}

impl UnitType {
    pub fn new(unit_type: UnitTypeEnum) -> Self {
        UnitType { unit_type }
    }
    pub fn unit_type(&self) -> UnitTypeEnum {
        self.unit_type
    }
}
impl Component for UnitType {}
