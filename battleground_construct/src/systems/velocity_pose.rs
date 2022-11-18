use super::components::pose::Pose;
use super::components::velocity::Velocity;
use super::Clock;
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
