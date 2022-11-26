use super::components;
use super::components::pose::Pose;
use super::Clock;
use engine::prelude::*;

pub struct FunctionPose {}
impl System for FunctionPose {
    fn update(&mut self, world: &mut World) {
        let (_entity, clock) = world
            .component_iter_mut::<Clock>()
            .next()
            .expect("Should have one clock");
        let t = clock.elapsed_as_f32();

        for (entity, fun) in world.component_iter::<components::function_pose::FunctionPose>() {
            if let Some(mut pose) = world.component_mut::<Pose>(&entity) {
                *pose = fun.pose(t)
            }
        }
    }
}
