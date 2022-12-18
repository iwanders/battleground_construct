use super::components;
use engine::prelude::*;

pub struct HealthCheck {}
impl System for HealthCheck {
    fn update(&mut self, world: &mut World) {
        let entity_health = world
            .component_iter::<components::health::Health>()
            .map(|(e, v)| (e, v.clone()))
            .collect::<Vec<(EntityId, components::health::Health)>>();
        for (entity, health) in entity_health {
            if health.is_destroyed() {
                world.add_component(entity, components::destroyed::Destroyed::new())
            }
        }
    }
}
