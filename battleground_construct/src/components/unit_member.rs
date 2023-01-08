use engine::prelude::*;
use serde::{Deserialize, Serialize};

use crate::components::unit::UnitId;

#[derive(Deserialize, Serialize, Debug, Clone, Copy)]
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
