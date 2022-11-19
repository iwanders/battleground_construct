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

impl std::ops::Deref for Pose {
    type Target = cgmath::Matrix4<f32>;
    fn deref(&self) -> &<Self as std::ops::Deref>::Target {
        &self.h
    }
}

impl From<cgmath::Matrix4<f32>> for Pose {
    fn from(v: cgmath::Matrix4<f32>) -> Self {
        Pose { h: v }
    }
}

impl Into<cgmath::Matrix4<f32>> for Pose {
    fn into(self) -> cgmath::Matrix4<f32> {
        self.h
    }
}

impl std::ops::Mul<Pose> for Pose {
    type Output = Pose;
    fn mul(self, v: Pose) -> <Self as std::ops::Mul<Pose>>::Output {
        Pose{h: self.h * v.h}
    }
}