use engine::prelude::*;

#[derive(Debug, Clone)]
pub struct DamageHit {
    damage: f32,
}

impl DamageHit {
    pub fn new(damage: f32) -> Self {
        DamageHit { damage }
    }
    pub fn damage(&self) -> f32 {
        self.damage
    }
}
impl Component for DamageHit {}
