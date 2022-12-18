use engine::prelude::*;

#[derive(Debug, Clone)]
pub struct Destroyed {}

impl Default for Destroyed {
    fn default() -> Self {
        Self::new()
    }
}

impl Destroyed {
    pub fn new() -> Self {
        Destroyed {}
    }
}
impl Component for Destroyed {}
