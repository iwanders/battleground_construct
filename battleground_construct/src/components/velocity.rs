use engine::prelude::*;

pub struct Velocity {
    /// Translation component.
    pub v: cgmath::Vector3<f32>,
    /// Rotation component.
    pub w: cgmath::Vector3<f32>,
}

impl Velocity {
    pub fn new() -> Self {
        Velocity {
            v: cgmath::Vector3::new(0.0, 0.0, 0.0),
            w: cgmath::Vector3::new(0.0, 0.0, 0.0),
        }
    }

    pub fn from_velocities(v: cgmath::Vector3<f32>, w: cgmath::Vector3<f32>) -> Self {
        Velocity { v, w }
    }

    pub fn integrate(&self, dt: f32) -> cgmath::Matrix4<f32> {
        cgmath::Matrix4::<f32>::from_translation(cgmath::Vector3::new(
            self.v[0] * dt,
            self.v[1] * dt,
            self.v[2] * dt,
        )) * cgmath::Matrix4::<f32>::from_angle_x(cgmath::Rad(self.w[0]))
            * cgmath::Matrix4::<f32>::from_angle_y(cgmath::Rad(self.w[1]))
            * cgmath::Matrix4::<f32>::from_angle_z(cgmath::Rad(self.w[2]))
    }

    pub fn integrate_pose(&self, pose: &super::pose::Pose, dt: f32) -> super::pose::Pose {
        (pose.h * self.integrate(dt)).into()
    }
}
impl Component for Velocity {}

#[cfg(test)]
mod test {
    use super::super::pose::Pose;
    use super::*;
    #[test]
    fn test_velocity_integration() {
        let start = Pose::new();
        let dt = 0.01f32;
        let mut v = Velocity::new();
        v.v[0] = 1.0;

        let mut p = start;
        for _i in 0..100 {
            p = (p.h * v.integrate(dt)).into();
        }

        assert!((p.h.w[0] - 100.0 * dt * 1.0).abs() <= 0.00001);
    }
}
