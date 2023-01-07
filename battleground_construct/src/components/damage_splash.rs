use engine::prelude::*;

#[derive(Debug, Clone)]
pub struct DamageSplash {
    damage: f32,
    radius: f32,
}

impl DamageSplash {
    pub fn new(damage: f32, radius: f32) -> Self {
        DamageSplash { damage, radius }
    }

    pub fn damage(&self) -> f32 {
        self.damage
    }

    pub fn radius(&self) -> f32 {
        self.radius
    }

    /// Returns the damage based on the distance, or None if no damage for this distance.
    pub fn damage_by_distance(&self, distance: f32) -> Option<f32> {
        if distance < self.radius {
            // Lets use a linear falloff right now.
            let ratio_of_distance = distance / self.radius;
            // Full damage at distance = 0, to 0 damage at distance self.radius.
            return Some((1.0 - ratio_of_distance) * self.damage);
        }
        None
    }
}
impl Component for DamageSplash {}
