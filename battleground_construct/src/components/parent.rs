use engine::prelude::*;

#[derive(Debug, Clone)]
pub struct Parent {
    pub parent: EntityId,
}

impl Parent {
    pub fn new(parent: EntityId) -> Self {
        Parent { parent }
    }
    pub fn parent(&self) -> &EntityId {
        &self.parent
    }
}
impl Component for Parent {}

impl From<Parent> for EntityId {
    fn from(v: Parent) -> EntityId {
        v.parent
    }
}
