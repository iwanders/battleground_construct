use super::components;
use engine::prelude::*;

pub struct Playback {}
impl System for Playback {
    fn update(&mut self, world: &mut World) {
        let recording = world
            .component_iter::<components::recorder::Recorder>()
            .next()
            .map(|v| v.1.recording());
        let finished = world
            .component_iter::<components::recorder::PlaybackFinishedMarker>()
            .next()
            .is_some();
        if !finished {
            if let Some(recording) = recording {
                recording.borrow_mut().step(world)
            }
        }
    }
}