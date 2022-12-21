use engine::prelude::*;
use crate::components::clock::Clock;
pub struct ClockSystem {}
impl System for ClockSystem {
    fn update(&mut self, world: &mut World) {
        let (_entity, mut clock) = world
            .component_iter_mut::<Clock>()
            .next()
            .expect("Should have one clock");
        clock.tick();
    }
}
