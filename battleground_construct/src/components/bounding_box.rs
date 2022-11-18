use engine::prelude::*;

#[derive(Copy, Debug, Clone)]
pub struct BoundingBox {
    pub min: cgmath::Vector3<f32>,
    pub max: cgmath::Vector3<f32>,
}

impl BoundingBox {
    pub fn new() -> Self {
        BoundingBox {
            min: cgmath::Vector3::new(-0.5, -0.5, -0.5),
            max: cgmath::Vector3::new(0.5, 0.5, 0.5),
        }
    }

    pub fn dimensions(&self) -> cgmath::Vector3<f32> {
        self.max - self.min
    }
}
impl Component for BoundingBox {}
