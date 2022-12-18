use engine::prelude::*;

#[derive(Debug, Clone, Copy)]
pub struct ProjectileSource {
    source: EntityId,
}

impl ProjectileSource {
    pub fn new(source: EntityId) -> Self {
        ProjectileSource { source }
    }

    pub fn source(&self) -> EntityId {
        self.source
    }
}
impl Component for ProjectileSource {}
