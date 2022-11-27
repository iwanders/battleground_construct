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
        let mut dead_root_entities = vec![];

        // Searching done, we can now do the logic.
        for (hit_entity, root_entity) in hit_entity_and_root {
            let (projectile_entity, source_entity) = {
                let projectile_entity = world.component::<HitBy>(hit_entity).unwrap();
                (projectile_entity.projectile(), projectile_entity.source())
            };
            let damage = world
                .component::<DamageDealer>(source_entity)
                .unwrap()
                .damage();
            let mut health = world.component_mut::<Health>(root_entity).unwrap();
            let _new_health = health.subtract(damage);
            // println!("New health: {new_health}");
            if health.is_dead() {
                // find the entire group.
                dead_root_entities.push(root_entity);
            }
            projectile_entities.push(projectile_entity);
        }

        // Salvage the particle generators on the particles.
        for particle_entity in projectile_entities.iter() {
            let particles_to_add = if let Some(p) =
                world.component::<super::display::particle_emitter::ParticleEmitter>(
                    *particle_entity,
                ) {
                Some(*p)
            } else {
                None
            };
            if let Some(mut copied_particle) = particles_to_add {
                copied_particle.emitting = false;
                let impact = world.add_entity();
                world.add_component(impact, super::components::expiry::Expiry::lifetime(5.0));
                world.add_component(impact, copied_particle);
            }
        }

        // The projectiles are now down, their hits are processed.
        world.remove_entities(&projectile_entities);

        // Now, we need to remove the HitBy from the hit entities.
        world.remove_components::<HitBy>(&hits);

        // Iterate through the dead root entities.
        let mut all_to_be_removed = vec![];
        for root_entity in dead_root_entities.iter() {
            let g = world.component::<Group>(*root_entity).unwrap();
            for part_entity in g.entities().iter().map(|x| *x) {
                all_to_be_removed.push(part_entity);
            }
        }
        world.remove_entities(&all_to_be_removed);
    }
}
