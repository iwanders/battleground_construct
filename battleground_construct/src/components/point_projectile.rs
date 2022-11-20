use engine::prelude::*;

#[derive(Debug, Clone)]
pub struct PointProjectile {
    source: EntityId,
}

impl PointProjectile {
    pub fn new(source: EntityId) -> Self {
        PointProjectile { source }
    }
    pub fn source(&self) -> EntityId {
        self.source
    }
}
impl Component for PointProjectile {}
