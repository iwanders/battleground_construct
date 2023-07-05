pub mod arm;
pub mod artillery;
pub mod base_tricycle;
pub mod capturable_flag;
pub mod common;
pub mod constructor;
pub mod tank;

use engine::prelude::*;

pub trait Unit {
    fn children(&self) -> Vec<EntityId>;
}
