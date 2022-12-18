use super::components;
use super::components::group::Group;
use super::components::health::Health;
use super::components::hit_by::HitBy;
use engine::prelude::*;

// Consumes HitBy components and handles logic of hitting a tank.
pub struct ProcessHitBy {}
impl System for ProcessHitBy {
    fn update(&mut self, world: &mut World) {
        let hits = world.component_entities::<HitBy>();

        // Find the root element of the HitBy elements.
        let hit_entity_and_root: Vec<(EntityId, EntityId)> = world
            .component_iter::<Group>()
            .filter(|(entity, _group)| hits.contains(entity))
            .map(|(entity, group)| (entity, group.entities()[0]))
            .collect::<_>();

        // Pop all HitBy objects from their components.
        let hit_by = world.remove_components::<HitBy>(&hits);

        // Ensure the roots have a HitByHistory
        for (_hit_entity, root_entity) in hit_entity_and_root.iter() {
            if world
                .component_mut::<components::hit_by::HitByHistory>(*root_entity)
                .is_none()
            {
                world.add_component(*root_entity, components::hit_by::HitByHistory::new());
            }
        }

        // Next, we can process the HitBy
        assert_eq!(hit_entity_and_root.len(), hit_by.len());
        for (ids, hit_by) in hit_entity_and_root.iter().zip(hit_by.iter()) {
            // let hit_entity = ids.0;
            let root_entity = ids.1;
            let hit_by = hit_by.as_ref().expect("all hits should have hitby now.");

            // Modify the health.
            if let Some(ref mut health) = world.component_mut::<Health>(root_entity) {
                for (damage, _impact) in hit_by.hits() {
                    health.subtract(damage);
                }
            }

            // Also add the hit to the HitByHistory.
            world
                .component_mut::<components::hit_by::HitByHistory>(root_entity)
                .expect("added above")
                .add_hits(hit_by);
        }
    }
}
