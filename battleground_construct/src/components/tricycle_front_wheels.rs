use engine::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct TricycleFrontWheels {
    wheels: Vec<EntityId>,
}

impl TricycleFrontWheels {
    pub fn new(wheels: &[EntityId]) -> Self {
        TricycleFrontWheels {
            wheels: wheels.to_vec(),
        }
    }
    pub fn wheels(&self) -> &[EntityId] {
        &self.wheels
    }
}
impl Component for TricycleFrontWheels {}
