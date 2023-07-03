pub mod arm;
pub mod artillery;
pub mod capturable_flag;
pub mod common;
pub mod tank;
pub mod constructor;

use engine::prelude::*;

pub trait Unit {
    fn children(&self) -> Vec<EntityId>;
}
