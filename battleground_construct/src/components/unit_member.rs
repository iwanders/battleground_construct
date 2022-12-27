use engine::prelude::*;

use crate::components::unit::UnitId;

#[derive(Debug, Clone, Copy)]
pub struct UnitMember {
    unit: UnitId,
}

impl UnitMember {
    pub fn new(unit: UnitId) -> Self {
        UnitMember { unit }
    }
    pub fn unit(&self) -> UnitId {
        self.unit
    }
}
impl Component for UnitMember {}
