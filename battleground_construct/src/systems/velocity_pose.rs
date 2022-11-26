use super::components::pose::Pose;
use super::components::velocity::Velocity;
use super::Clock;
use crate::util::cgmath::ToQuaternion;
use engine::prelude::*;

pub struct VelocityPose {}
impl System for VelocityPose {
    fn update(&mut self, world: &mut World) {
        let (_entity, clock) = world
            .component_iter_mut::<Clock>()
            .next()
            .expect("Should have one clock");
        let dt = clock.step_as_f32();

        for (entity, vel) in world.component_iter::<Velocity>() {
            // try to see if we can find a velocity for this entity.
            if let Some(mut pose) = world.component_mut::<Pose>(&entity) {
                // Yes, so now integrate it.
                *pose = vel.integrate_pose(&pose, dt);
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_integration_of_quaternion() {
        let dt = 0.001;
        let vel = Velocity::from_se2(0.45, 0.0, -0.3);
        let mut pose = Pose::from_xyz(100.0, 100.0, 0.0);
        let t_max = 700.0;
        let mut t = 0.0;

        fn angle(p: &Pose) -> f32 {
            p.transform().x[1].atan2(p.transform().x[0])
        }
        type Mat4 = cgmath::Matrix4<f32>;

        while t < t_max {
            let previous_pose = pose;
            pose = vel.integrate_pose(&pose, dt);
            t += dt;

            use cgmath::MetricSpace;
            use cgmath::SquareMatrix;

            // Check that the rotation part of the matrix is still correct.
            let previous_m3x3 = cgmath::Matrix3::<f32>::from_cols(
                pose.x.truncate(),
                pose.y.truncate(),
                pose.z.truncate(),
            );
            let mut transposed = previous_m3x3;
            transposed.transpose_self();
            let res = transposed * previous_m3x3;
            let res_diag = res.diagonal();
            let expected = cgmath::Vector3::<f32>::new(1.0, 1.0, 1.0);
            let dist = res_diag.distance2(expected);
            let det = previous_m3x3.determinant();
            assert!(
                dist < 0.0001,
                "Distance between rotation * rotation.T diagonal exceeps 0.001: {dist:?}, {res:?}"
            );
            assert!(
                (det - 1.0).abs() < 0.001,
                "Determinant error exceeds 0.001: {det}"
            );
        }
    }
}
