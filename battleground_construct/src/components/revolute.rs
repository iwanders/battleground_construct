use engine::prelude::*;
use crate::components::pose::Pose;


#[derive(Copy, Debug, Clone)]
pub struct Revolute {
    pub velocity_bounds: (f32, f32),
    pub velocity: f32,
    pub position: f32,
    pub transform: cgmath::Matrix4::<f32>,
}

impl Revolute {
    pub fn new() -> Self {
        Self::new_with_transform(cgmath::Matrix4::<f32>::from_scale(1.0))
    }
    pub fn new_with_transform(transform: cgmath::Matrix4::<f32>) -> Self {
        Revolute {
            velocity: 0.0,
            position: 0.0,
            velocity_bounds: (-1.0, 1.0),
            transform,
        }
    }

    pub fn set_velocity(&mut self, velocity: f32) {
        self.velocity =  velocity.clamp(self.velocity_bounds.0, self.velocity_bounds.1);
    }

    pub fn velocity(&self) -> f32 {
        self.velocity
    }

    pub fn position(&self) -> f32 {
        self.position
    }

    pub fn velocity_bounds(&mut self) -> (f32, f32) {
        self.velocity_bounds
    }

    pub fn integrate(&mut self, dt: f32){
        self.position += (dt * self.velocity).rem_euclid(std::f32::consts::PI * 2.0);
    }

    pub fn to_pose(&self) -> Pose {
        (cgmath::Matrix4::<f32>::from_angle_x(cgmath::Rad(self.position)) * self.transform).into()
    }
}
impl Component for Revolute {}
