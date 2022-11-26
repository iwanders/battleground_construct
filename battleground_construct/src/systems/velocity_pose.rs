use super::components::pose::Pose;
use super::components::velocity::Velocity;
use super::Clock;
use engine::prelude::*;
use crate::util::cgmath::ToQuaternion;

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
mod test{
    use super::*;
    #[test]
    fn test_integration_of_quaternion() {
        let dt = 0.001;
        let vel = Velocity::from_se2(0.45, 0.0, -0.3);
        let mut pose = Pose::from_xyz(100.0, 100.0, 0.0);
        let t_max = 700.0;
        let mut t = 0.0;
        while t < t_max {
            let previous_pose = *pose;
            pose = vel.integrate_pose(&pose, dt);
            t += dt;
            if t > 600.0 {
                let diff = previous_pose.w.truncate() - (*pose).w.truncate();
                let dist = diff.x.powf(2.0) + diff.y.powf(2.0) + diff.z.powf(2.0);
                let pq = previous_pose.to_quaternion();
                let q = pose.to_quaternion();
                let dq = pq - q;
                let qdist = dq.v.x.powf(2.0) + dq.v.y.powf(2.0) + dq.v.z.powf(2.0) + dq.s.powf(2.0);
                // println!("vel: {vel:?}");
                if (qdist > 2.5e-8 || dist > 2.1e-7){
                    println!("vel: {vel:?}");
                    println!("dist: {dist:?}");
                    println!("Previous pose: {previous_pose:?}");
                    println!("New pose: {pose:?}");
                    println!("q: {q:?}");
                    println!("dq: {dq:?}");
                    println!("qdist: {qdist:?}");
                }
            }
        }
    }
}
