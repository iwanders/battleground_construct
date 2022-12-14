use engine::prelude::*;

#[derive(Debug, Clone)]
pub struct RadarReflector {
    pub reflectivity: f32,
}
impl Default for RadarReflector {
    fn default() -> Self {
        RadarReflector::new()
    }
}

impl RadarReflector {
    pub fn new() -> Self {
        Self { reflectivity: 1.0 }
    }
    pub fn reflectivity(&self) -> f32 {
        self.reflectivity
    }
}
impl Component for RadarReflector {}
