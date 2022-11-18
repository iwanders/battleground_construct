use engine::prelude::*;

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
}
impl Component for BoundingBox {}

