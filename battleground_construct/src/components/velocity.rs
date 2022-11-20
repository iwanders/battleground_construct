use engine::prelude::*;

#[derive(Copy, Debug, Clone)]
pub struct Velocity {
    /// Translation component.
    pub v: cgmath::Vector3<f32>,
    /// Rotation component.
    pub w: cgmath::Vector3<f32>,
}

#[derive(Copy, Debug, Clone)]
pub struct GlobalVelocity {
    /// Translation component.
    pub v: cgmath::Vector3<f32>,
    /// Rotation component.
    pub w: cgmath::Vector3<f32>,
}

macro_rules! create_velocity_implementation {
    ($the_type:ty) => {
        impl $the_type {
            pub fn new() -> Self {
                Self {
                    v: cgmath::Vector3::new(0.0, 0.0, 0.0),
                    w: cgmath::Vector3::new(0.0, 0.0, 0.0),
                }
            }
            pub fn from_se2(x: f32, y: f32, yaw: f32) -> Self {
                Self {
                    v: cgmath::Vector3::new(x, y, 0.0),
                    w: cgmath::Vector3::new(0.0, 0.0, yaw),
                }
            }

            pub fn from_velocities(v: cgmath::Vector3<f32>, w: cgmath::Vector3<f32>) -> Self {
                Self { v, w }
            }

            pub fn integrate(&self, dt: f32) -> cgmath::Matrix4<f32> {
                cgmath::Matrix4::<f32>::from_translation(cgmath::Vector3::new(
                    self.v[0] * dt,
                    self.v[1] * dt,
                    self.v[2] * dt,
                )) * cgmath::Matrix4::<f32>::from_angle_x(cgmath::Rad(self.w[0]) * dt)
                    * cgmath::Matrix4::<f32>::from_angle_y(cgmath::Rad(self.w[1]) * dt)
                    * cgmath::Matrix4::<f32>::from_angle_z(cgmath::Rad(self.w[2]) * dt)
            }

            pub fn integrate_pose(&self, pose: &super::pose::Pose, dt: f32) -> super::pose::Pose {
                (pose.h * self.integrate(dt)).into()
            }
            pub fn integrate_global_pose(
                &self,
                pose: &super::pose::Pose,
                dt: f32,
            ) -> super::pose::Pose {
                (self.integrate(dt) * pose.h).into()
            }
        }
        impl Component for $the_type {}
    };
}
create_velocity_implementation!(Velocity);
// create_velocity_implementation!(GlobalVelocity);

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
