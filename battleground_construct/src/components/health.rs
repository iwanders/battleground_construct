use engine::prelude::*;

#[derive(Debug, Clone)]
pub struct Health {
    pub health: f32,
}

impl Health {
    pub fn new() -> Self {
        Health { health: 1.0 }
    }
    pub fn health(&self) -> f32 {
        self.health
    }
}
impl Component for Health {}
