use engine::prelude::*;

#[derive(Copy, Debug, Clone)]
pub struct RevolutePair {
    pair: EntityId,
    scale: f32,
}

impl RevolutePair {
    pub fn new(pair: EntityId, scale: f32) -> Self {
        Self { pair, scale }
    }
    pub fn pair(&self) -> EntityId {
        self.pair
    }

    pub fn scale(&self) -> f32 {
        self.scale
    }
}
impl Component for RevolutePair {}
