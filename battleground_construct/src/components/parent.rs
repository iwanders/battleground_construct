use engine::prelude::*;

#[derive(Debug, Clone)]
pub struct Parent {
    pub parent: EntityId
}

impl Parent {
    pub fn new(parent: EntityId) -> Self {
        Parent {
            parent
        }
    }
}
impl Component for Parent {}

