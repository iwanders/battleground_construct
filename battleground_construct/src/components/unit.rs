use engine::prelude::*;
use serde::{Deserialize, Serialize};

pub use battleground_unit_control::units::UnitType;

#[derive(Deserialize, Serialize, Copy, Clone, Debug, Eq, Ord, PartialOrd, PartialEq, Hash)]
pub struct UnitId(u64);

impl UnitId {
    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

#[cfg(test)]
pub fn make_unit_id(v: u64) -> UnitId {
    UnitId(v)
}

#[derive(Debug, Clone)]
pub struct Unit {
    id: UnitId,
    unit_type: UnitType,
}

impl Unit {
    pub fn new(id: u64, unit_type: UnitType) -> Self {
        Unit {
            id: UnitId(id),
            unit_type,
        }
    }

    pub fn id(&self) -> UnitId {
        self.id
    }

    pub fn unit_type(&self) -> UnitType {
        self.unit_type
    }
}
impl Component for Unit {}

use crate::components::unit_interface::{Register, RegisterMap, UnitModule};

#[derive(Debug, Clone, Copy)]
pub struct UnitModuleComponent {
    unit_entity: EntityId,
}

impl UnitModuleComponent {
    pub fn new(unit_entity: EntityId) -> Self {
        UnitModuleComponent { unit_entity }
    }
}
impl Component for UnitModuleComponent {}

impl UnitModule for UnitModuleComponent {
    fn get_registers(&self, world: &World, registers: &mut RegisterMap) {
        let value = world
            .component::<Unit>(self.unit_entity)
            .and_then(|t| Some(t.id().as_u64() as i32))
            .unwrap_or(battleground_unit_control::modules::unit::UNIT_NO_UNIT_ID);
        registers.insert(
            battleground_unit_control::modules::unit::REG_UNIT_UNIT_ID,
            Register::new_i32("unit_id", value),
        );
        let value = world
            .component::<Unit>(self.unit_entity)
            .and_then(|t| Some(t.unit_type() as i32))
            .unwrap_or(battleground_unit_control::modules::unit::UNIT_NO_UNIT_TYPE);
        registers.insert(
            battleground_unit_control::modules::unit::REG_UNIT_UNIT_TYPE,
            Register::new_i32("unit_type", value),
        );
    }
}

pub fn get_unit_entity(world: &World, unit_id: UnitId) -> Option<EntityId> {
    for (entity, v) in world.component_iter::<Unit>() {
        if v.id() == unit_id {
            return Some(entity);
        }
    }
    None
}
