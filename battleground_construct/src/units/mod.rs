pub mod artillery;
pub mod capturable_flag;
pub mod common;
pub mod arm;
pub mod tank;

use engine::prelude::*;

pub trait Unit {
    fn children(&self) -> Vec<EntityId>;
}
