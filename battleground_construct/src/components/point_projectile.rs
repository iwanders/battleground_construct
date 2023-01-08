use engine::prelude::*;

#[derive(Debug, Copy, Clone)]
pub struct PointProjectile {}

impl Default for PointProjectile {
    fn default() -> Self {
        Self::new()
    }
}

impl PointProjectile {
    pub fn new() -> Self {
        PointProjectile {}
    }
}
impl Component for PointProjectile {}
