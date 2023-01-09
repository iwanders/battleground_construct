use super::components;
use engine::prelude::*;

pub struct Playback {}
impl System for Playback {
    fn update(&mut self, world: &mut World) {
        let record = world
            .component_iter::<components::recording::Recording>()
            .next()
            .map(|v| v.1.record());
        let finished = world
            .component_iter::<components::recording::PlaybackFinishedMarker>()
            .next()
            .is_some();
        if !finished {
            if let Some(record) = record {
                record.borrow_mut().step(world)
            }
        }
    }
}
