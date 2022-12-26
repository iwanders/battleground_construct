use crate::components;
use components::clock::Clock;
use components::match_time_limit::MatchTimeLimit;

use engine::prelude::*;

pub struct MatchLogicTimeLimit {}
impl System for MatchLogicTimeLimit {
    fn update(&mut self, world: &mut World) {
        let t = world
            .component_iter_mut::<Clock>()
            .next()
            .expect("Should have one clock")
            .1
            .elapsed_as_f32();

        if let Some((_e, mut time_limit)) = world.component_iter_mut::<MatchTimeLimit>().next() {
            time_limit.set_time(t);
        }
    }
}
