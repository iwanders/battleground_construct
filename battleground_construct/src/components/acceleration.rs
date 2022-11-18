use engine::prelude::*;

pub struct Acceleration {
    /// Translation component.
    pub dv: cgmath::Vector3<f32>,
    /// Rotation component.
    pub dw: cgmath::Vector3<f32>,
}

impl Acceleration {
    pub fn new() -> Self {
        Acceleration {
            dv: cgmath::Vector3::new(0.0, 0.0, 0.0),
            dw: cgmath::Vector3::new(0.0, 0.0, 0.0),
        }
    }

    pub fn integrate_velocity(
        &self,
        velocity: &super::velocity::Velocity,
        dt: f32,
    ) -> super::velocity::Velocity {
        super::velocity::Velocity::from_velocities(
            self.dv * dt + velocity.v,
            self.dw * dt + velocity.w,
        )
    }
}
impl Component for Acceleration {}
