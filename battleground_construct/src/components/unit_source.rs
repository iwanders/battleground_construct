use engine::prelude::*;
use super::unit::UnitId;

#[derive(Debug, Clone, Copy)]
pub struct UnitSource {
    source: UnitId,
}

impl UnitSource {
    pub fn new(source: UnitId) -> Self {
        UnitSource { source }
    }

    pub fn source(&self) -> UnitId {
        self.source
    }
}
impl Component for UnitSource {}
