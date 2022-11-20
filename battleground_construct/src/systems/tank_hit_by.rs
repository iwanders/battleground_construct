use super::components::damage_dealer::DamageDealer;
use super::components::group::Group;
use super::components::health::Health;
use super::components::hit_by::HitBy;
use engine::prelude::*;

// Consumes HitBy components and handles logic of hitting a tank.
pub struct TankHitBy {}
impl System for TankHitBy {
    fn update(&mut self, world: &mut World) {
        let hits = world.component_entities::<HitBy>();
        // Find the root element of those.
        let hit_entity_and_root: Vec<(EntityId, EntityId)> = world
            .component_iter::<Group>()
            .filter(|(entity, _group)| hits.contains(entity))
            .map(|(entity, group)| (entity, group.entities()[0]))
            .collect::<_>();

        let mut projectile_entities = vec![];

        // Searching done, we can now do the logic.
        for (hit_entity, root_entity) in hit_entity_and_root {
            let (projectile_entity, source_entity) = {
                let projectile_entity = world.component::<HitBy>(&hit_entity).unwrap();
                (projectile_entity.projectile(), projectile_entity.source())
            };
            let damage = world
                .component::<DamageDealer>(&source_entity)
                .unwrap()
                .damage();
            let mut health = world.component_mut::<Health>(&root_entity).unwrap();
            let new_health = health.subtract(damage);
            println!("New health: {new_health}");
            projectile_entities.push(projectile_entity);
        }

        // The projectiles are now down, their hits are processed.
        world.remove_entities(&projectile_entities);
        // Now, we need to remove the HitBy from the hit entities.
        world.remove_components::<HitBy>(&hits);
    }
}