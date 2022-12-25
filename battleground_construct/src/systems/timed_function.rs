use super::components::clock::Clock;
use super::components::timed_function_trigger::TimedFunctionTrigger;

use engine::prelude::*;

pub struct TimedFunction {}
impl System for TimedFunction {
    fn update(&mut self, world: &mut World) {
        let t = world
            .component_iter_mut::<Clock>()
            .next()
            .expect("Should have one clock")
            .1
            .elapsed_as_f32();

        let mut to_be_removed = vec![];
        let mut to_be_called = vec![];

        // Obtain all the functions for which we are to do something.
        for (entity, mut trigger) in world.component_iter_mut::<TimedFunctionTrigger>() {
            if let Some(f) = trigger.should_call(t) {
                to_be_called.push((entity, f));
            }
            if trigger.is_done() {
                to_be_removed.push(entity);
            }
        }

        // Call the functions.
        for (e, f) in to_be_called {
            f(e, world);
        }

        // Now, remove all the components that are done..
        world.remove_components::<TimedFunctionTrigger>(&to_be_removed);
    }
}
