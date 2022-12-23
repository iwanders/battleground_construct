use crate::display;
use engine::prelude::*;

#[derive(Debug, Clone)]
pub struct Team {
    name: String,
    color: display::Color,
}

impl Team {
    pub fn new(name: &str, color: display::Color) -> Self {
        Team {
            name: name.to_owned(),
            color,
        }
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn color(&self) -> &display::Color {
        &self.color
    }
}
impl Component for Team {}
