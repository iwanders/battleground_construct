use engine::prelude::*;

#[derive(Copy, Debug, Clone)]
pub struct HitPlane {
    pub direction: cgmath::Vector3<f32>,
}

impl Default for HitPlane {
    fn default() -> Self {
        HitPlane::new()
    }
}

impl HitPlane {
    pub fn new() -> Self {
        HitPlane {
            direction: cgmath::Vector3::<f32>::new(0.0, 0.0, 1.0),
        }
    }
    pub fn above(&self, v: cgmath::Vector3<f32>) -> bool {
        cgmath::dot(self.direction, v) < 0.0
    }
}
impl Component for HitPlane {}
