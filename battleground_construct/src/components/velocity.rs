use engine::prelude::*;

pub struct Velocity {
    /// Translation component.
    pub v: cgmath::Vector3<f32>,
    /// Rotation component.
    pub w: cgmath::Vector3<f32>
}

impl Velocity {
    pub fn new() -> Self{
        Velocity {
            v: cgmath::Vector3::new(0.0, 0.0, 0.0),
            w: cgmath::Vector3::new(0.0, 0.0, 0.0),
        }
    }

    pub fn integrate(&self, dt: f32) -> cgmath::Matrix4<f32> {
        cgmath::Matrix4::<f32>::from_translation(cgmath::Vector3::new(self.v[0] * dt, self.v[1] * dt, self.v[2] * dt))
        * cgmath::Matrix4::<f32>::from_angle_x(cgmath::Rad(self.w[0]))
        * cgmath::Matrix4::<f32>::from_angle_y(cgmath::Rad(self.w[1]))
        * cgmath::Matrix4::<f32>::from_angle_z(cgmath::Rad(self.w[2]))
    }
}
impl Component for Velocity {}

#[cfg(test)]
mod test {
    use super::*;
    use super::super::pose::Pose;
    #[test]
    fn test_velocity_integration() {
        let start = Pose::new();
        let dt = 0.01f32;
        let mut v = Velocity::new();
        v.v[0] = 1.0;
        // v.w[2] = 1.0;
        
        let mut p = start;
        for i in 0..100 {
            p = (p.H * v.integrate(dt)).into();
        }
        assert_eq!(p.H.w[0], 100.0 * dt * 1.0);
    }

}