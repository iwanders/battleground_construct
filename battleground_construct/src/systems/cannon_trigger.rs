use super::components::cannon::Cannon;

use super::Clock;
use engine::prelude::*;

pub struct CannonTrigger {}
impl System for CannonTrigger {
    fn update(&mut self, world: &mut World) {
        let current = {
            let (_entity, clock) = world
                .component_iter_mut::<Clock>()
                .next()
                .expect("Should have one clock");
            clock.elapsed_as_f32()
        };

        for cannon_entity in world.component_entities::<Cannon>() {
            let fired = {
                let mut cannon = world.component_mut::<Cannon>(cannon_entity).unwrap();
                cannon.update(current);
                if cannon.is_triggered() && cannon.is_ready() {
                    cannon.fired(current);
                    true
                } else {
                    false
                }
            };

            if !fired {
                continue;
            }

            let cannon_effect = { world.component::<Cannon>(cannon_entity).unwrap().effect() };

            cannon_effect(world, cannon_entity);
        }
    }
}
