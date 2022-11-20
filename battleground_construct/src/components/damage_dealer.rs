use engine::prelude::*;

#[derive(Debug, Clone)]
pub struct DamageDealer {
    damage: f32
}

impl DamageDealer {
    pub fn new(damage: f32) -> Self {
        DamageDealer {
            damage
        }
    }
    pub fn damage(&self) -> f32 {
        self.damage
    }
}
impl Component for DamageDealer {}
