use crate::components::pose::Pose;
use engine::prelude::*;

#[derive()]
pub struct FunctionPose {
    pub fun: Box<dyn Fn(f32) -> Pose>,
}

impl FunctionPose {
    pub fn new<F>(fun: F) -> Self
    where
        F: Fn(f32) -> Pose + 'static,
    {
        FunctionPose { fun: Box::new(fun) }
    }

    pub fn pose(&self, time: f32) -> Pose {
        (self.fun)(time)
    }
}
impl Component for FunctionPose {}
