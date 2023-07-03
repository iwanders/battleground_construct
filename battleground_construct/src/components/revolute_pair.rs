use engine::prelude::*;

#[derive(Copy, Debug, Clone)]
pub struct RevolutePair {
    pair: EntityId,
}

impl RevolutePair {
    pub fn new(pair: EntityId) -> Self {
        Self { pair }
    }
    pub fn pair(&self) -> EntityId {
        self.pair
    }
}
impl Component for RevolutePair {}
