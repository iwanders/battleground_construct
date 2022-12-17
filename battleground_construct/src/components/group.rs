use engine::prelude::*;

#[derive(Debug, Clone, Default)]
/// First entity is always the 'master' entity in the group, this can be used to find the common
/// components
pub struct Group {
    pub entities: Vec<EntityId>,
}

impl Group {
    pub fn new() -> Self {
        Group::default()
    }
    pub fn from(entities: &[EntityId]) -> Self {
        Group {
            entities: entities.to_vec(),
        }
    }
    pub fn entities(&self) -> &[EntityId] {
        &self.entities[..]
    }
}
impl Component for Group {}
