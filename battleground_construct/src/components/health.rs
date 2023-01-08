use engine::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Health {
    pub health: f32,
}
impl Default for Health {
    fn default() -> Self {
        Health::new()
    }
}

impl Health {
    pub fn new() -> Self {
        Health { health: 1.0 }
    }

    pub fn health(&self) -> f32 {
        self.health
    }

    pub fn subtract(&mut self, value: f32) -> f32 {
        self.health -= value;
        self.health
    }

    pub fn is_destroyed(&self) -> bool {
        self.health <= 0.0
    }
}
impl Component for Health {}
