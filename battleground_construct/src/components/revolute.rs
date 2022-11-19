use engine::prelude::*;
use crate::components::pose::Pose;


#[derive(Copy, Debug, Clone)]
pub struct Revolute {
    pub velocity_bounds: (f32, f32),
    pub velocity: f32,
    pub position: f32,
    pub axis: cgmath::Vector3<f32>,
}

impl Revolute {
    pub fn new() -> Self {
        Self::new_with_axis(cgmath::Vector3::<f32>::new(1.0, 0.0, 0.0))
    }
    pub fn new_with_axis(axis: cgmath::Vector3::<f32>) -> Self {
        Revolute {
            velocity: 0.0,
            position: 0.0,
            axis,
            velocity_bounds: (-1.0, 1.0),
        }
    }

    pub fn set_axis(&mut self, axis:cgmath::Vector3::<f32>) {
        self.axis = axis;
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
        self.position = (self.position + dt * self.velocity).rem_euclid(std::f32::consts::PI * 2.0);
    }

    pub fn to_pose(&self) -> Pose {
        cgmath::Matrix4::<f32>::from_axis_angle(self.axis, cgmath::Rad(self.position)).into()
    }
}
impl Component for Revolute {}
