use engine::prelude::*;

pub use battleground_unit_control::units::UnitType;

#[derive(Copy, Clone, Debug, Eq, Ord, PartialOrd, PartialEq, Hash)]
pub struct UnitId(u64);

impl UnitId {
    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

#[cfg(test)]
pub fn make_unit_id(v: u64) -> TeamId {
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

pub fn get_unit_entity(world: &World, unit_id: UnitId) -> Option<EntityId> {
    for (entity, v) in world.component_iter::<Unit>() {
        if v.id() == unit_id {
            return Some(entity);
        }
    }
    None
}
