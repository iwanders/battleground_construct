use super::components;
use engine::prelude::*;

pub struct Record {}
impl System for Record {
    fn update(&mut self, world: &mut World) {
        let recording = world
            .component_iter::<components::recorder::Recorder>()
            .next()
            .map(|v| v.1.recording());
        if let Some(recording) = recording {
            recording.borrow_mut().record(world)
        }
    }
}
