use engine::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct TricycleFrontWheel {
    wheels: Vec<EntityId>,
}

impl TricycleFrontWheel {
    pub fn new(wheels: &[EntityId]) -> Self {
        TricycleFrontWheel {
            wheels: wheels.to_vec(),
        }
    }
    pub fn wheels(&self) -> &[EntityId] {
        &self.wheels
    }
}
impl Component for TricycleFrontWheel {}
