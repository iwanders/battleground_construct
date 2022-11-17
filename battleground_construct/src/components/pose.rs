use engine::prelude::*;

pub struct Pose {
    pub H: cgmath::Matrix4<f32>
}

impl Pose {
    pub fn new() -> Self{
        Pose {
            H: cgmath::Matrix4::<f32>::from_translation(cgmath::Vector3::new(0.0, 0.0, 0.0))
        }
    }
}
impl Component for Pose {}

impl From<cgmath::Matrix4::<f32>> for Pose {
    fn from(v: cgmath::Matrix4::<f32>) -> Self {
        Pose{H: v}
    }
}

