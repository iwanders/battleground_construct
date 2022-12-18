use engine::prelude::*;

#[derive(Debug, Clone)]
pub struct Destroyed {}

impl Destroyed {
    pub fn new() -> Self {
        Destroyed {}
    }
}
impl Component for Destroyed {}
