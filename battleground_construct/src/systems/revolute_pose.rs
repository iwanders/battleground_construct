use super::components::pose::Pose;
use super::components::revolute::Revolute;
use super::Clock;
use engine::prelude::*;

pub struct RevolutePose {}
impl System for RevolutePose {
    fn update(&mut self, world: &mut World) {
        let (_entity, clock) = world
            .component_iter_mut::<Clock>()
            .next()
            .expect("Should have one clock");
        let dt = clock.step_as_f32();

        for (entity, mut rev) in world.component_iter_mut::<Revolute>() {
            rev.integrate(dt);
            // try to see if we can find a velocity for this entity.
            if let Some(mut pose) = world.component_mut::<Pose>(&entity) {
                // Yes, so now integrate it.
                *pose = rev.to_pose().into()
            }
        }
    }
}
