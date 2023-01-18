use engine::prelude::*;

#[derive(Debug, Clone)]
pub struct RadarReflector {
    pub reflectivity: f32,
}

impl RadarReflector {
    pub fn new(reflectivity: f32) -> Self {
        Self { reflectivity }
    }
    pub fn reflectivity(&self) -> f32 {
        self.reflectivity
    }
}
impl Component for RadarReflector {}
