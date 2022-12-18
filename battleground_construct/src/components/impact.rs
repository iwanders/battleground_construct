use engine::prelude::*;

#[derive(Debug, Clone)]
pub struct Impact {
    impact_on: Option<EntityId>,
    source: EntityId,
    position: cgmath::Matrix4<f32>,
}

impl Impact {
    pub fn new(
        impact_on: Option<EntityId>,
        position: cgmath::Matrix4<f32>,
        source: EntityId,
    ) -> Self {
        Impact {
            impact_on,
            position,
            source,
            // impact,
        }
    }

    pub fn impact_on(&self) -> Option<EntityId> {
        self.impact_on
    }

    pub fn position(&self) -> cgmath::Matrix4<f32> {
        self.position
    }
    pub fn source(&self) -> EntityId {
        self.source
    }
}
impl Component for Impact {}
