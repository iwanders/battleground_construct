use engine::prelude::*;

#[derive(Debug, Clone)]
pub struct HitBy {
    projectile: EntityId,
    source: EntityId,
    impact: cgmath::Matrix4<f32>,
}

impl HitBy {
    pub fn new(projectile: EntityId, source: EntityId, impact: cgmath::Matrix4<f32>) -> Self {
        HitBy { projectile, source, impact }
    }
    pub fn source(&self) -> EntityId {
        self.source
    }
    pub fn projectile(&self) -> EntityId {
        self.projectile
    }
    pub fn impact(&self) -> cgmath::Matrix4<f32> {
        self.impact
    }
}
impl Component for HitBy {}
