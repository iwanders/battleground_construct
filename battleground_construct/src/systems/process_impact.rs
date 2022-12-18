use crate::components;
use engine::prelude::*;

pub struct ProcessImpact {}
impl System for ProcessImpact {
    fn update(&mut self, world: &mut World) {
        let t = world
            .component_iter::<components::clock::Clock>()
            .next()
            .expect("Should have one clock")
            .1
            .elapsed_as_f32();

        // Iterate over all Impact objects.
        // Consume DamageHit and DamageSplash from those entities.
        // Update other 'HitBy' records.

        // Get all projectiles' world poses.
        let impacts = world.component_entities::<components::impact::Impact>();

        // Lets just process DamageHit for now.

        // Collect all DamageHit, Impact and ProjectileSource objects.
        for impact_entity in impacts.iter() {
            let impact_entity = *impact_entity;
            // Clone the relevant entities, this allows us to mutably borrow the world again.
            let impact = world
                .component::<components::impact::Impact>(impact_entity)
                .expect("only considering impacts")
                .clone();
            let damage_hit = world
                .component::<components::damage_hit::DamageHit>(impact_entity)
                .map(|v| v.clone());
            let projectile_source = world
                .component::<components::projectile_source::ProjectileSource>(impact_entity)
                .map(|v| *v)
                .expect("projectiles should have source");

            if let Some(damage_hit) = damage_hit {
                if let Some(impact_on) = impact.impact_on() {
                    // Add HitBy or retrieve.
                    if world
                        .component_mut::<components::hit_by::HitBy>(impact_on)
                        .is_none()
                    {
                        world.add_component(impact_on, components::hit_by::HitBy::new());
                    }
                    // Now, we can add a record to the HitBy component.
                    let mut hit_by = world
                        .component_mut::<components::hit_by::HitBy>(impact_on)
                        .unwrap();
                    hit_by.add_hit(damage_hit, impact, projectile_source.source(), t);
                }
            }
        }

        // Remove all entities;
        world.remove_entities(&impacts);
    }
}
