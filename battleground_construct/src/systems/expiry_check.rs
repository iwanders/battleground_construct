use super::components::clock::Clock;
use super::components::expiry::Expiry;

use engine::prelude::*;

pub struct ExpiryCheck {}
impl System for ExpiryCheck {
    fn update(&mut self, world: &mut World) {
        let t = world
            .component_iter_mut::<Clock>()
            .next()
            .expect("Should have one clock")
            .1
            .elapsed_as_f32();
        let mut to_be_removed = vec![];

        for (entity, mut expiry) in world.component_iter_mut::<Expiry>() {
            expiry.update_expiry(t);
            if expiry.is_expired(t) {
                to_be_removed.push(entity);
            }
        }

        // Now, remove all these entities.
        world.remove_entities(&to_be_removed);
    }
}
