use super::components;
use engine::prelude::*;

pub struct Record {}
impl System for Record {
    fn update(&mut self, world: &mut World) {
        let record = world
            .component_iter::<components::recording::Recording>()
            .next()
            .map(|v| v.1.record());
        if let Some(record) = record {
            record.borrow_mut().record(world)
        }
    }
}
