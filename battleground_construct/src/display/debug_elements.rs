use super::primitives::*;
use engine::prelude::*;

#[derive(Debug, Clone, Default)]
pub struct DebugElements {
    elements: Vec<Element>,
}

impl DebugElements {
    pub fn new() -> Self {
        DebugElements::default()
    }
    pub fn add_element(&mut self, element: Element) {
        self.elements.push(element)
    }
}
impl Component for DebugElements {}

impl Drawable for DebugElements {
    fn drawables(&self) -> Vec<Element> {
        self.elements.clone()
    }
}
