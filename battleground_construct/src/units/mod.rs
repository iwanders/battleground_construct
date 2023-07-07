pub mod arm;
pub mod artillery;
pub mod base_tricycle;
pub mod capturable_flag;
pub mod common;
pub mod constructor;
pub mod tank;

pub use crate::components::unit::UnitId;

use engine::prelude::*;

pub trait Unit {
    fn children(&self) -> Vec<EntityId>;
    fn unit_entity(&self) -> EntityId;
    fn unit_id(&self) -> UnitId;
}
