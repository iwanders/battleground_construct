use engine::prelude::*;

#[derive(Debug, Clone)]
pub struct HitBy {
    projectile: EntityId,
    source: EntityId,
}

impl HitBy {
    pub fn new(projectile: EntityId, source: EntityId) -> Self {
        HitBy { projectile, source }
    }
    pub fn source(&self) -> EntityId {
        self.source
    }
    pub fn projectile(&self) -> EntityId {
        self.projectile
    }
}
impl Component for HitBy {}
