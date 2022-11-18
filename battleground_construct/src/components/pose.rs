use engine::prelude::*;

#[derive(Copy, Debug, Clone)]
pub struct Pose {
    pub h: cgmath::Matrix4<f32>,
}

impl Pose {
    pub fn new() -> Self {
        Pose {
            h: cgmath::Matrix4::<f32>::from_translation(cgmath::Vector3::new(0.0, 0.0, 0.0)),
        }
    }
}
impl Component for Pose {}

impl From<cgmath::Matrix4<f32>> for Pose {
    fn from(v: cgmath::Matrix4<f32>) -> Self {
        Pose { h: v }
    }
}
