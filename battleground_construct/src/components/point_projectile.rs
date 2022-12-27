use engine::prelude::*;

#[derive(Debug, Copy, Clone)]
pub struct PointProjectile {}

impl PointProjectile {
    pub fn new() -> Self {
        PointProjectile { }
    }
}
impl Component for PointProjectile {}
