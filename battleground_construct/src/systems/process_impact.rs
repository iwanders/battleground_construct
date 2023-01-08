use crate::components;
use crate::util::box_collision::AxisAlignedBox;
use crate::util::cgmath::prelude::*;
use components::hit_box::HitBox;
use components::hit_collection::HitCollection;
use components::pose::world_pose;
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
            let impact_pose = impact.position();
            let damage_hit = world
                .component::<components::damage_hit::DamageHit>(impact_entity)
                .map(|v| v.clone());
            let damage_splash = world
                .component::<components::damage_splash::DamageSplash>(impact_entity)
                .map(|v| v.clone());
            let unit_source = world
                .component::<components::unit_source::UnitSource>(impact_entity)
                .map(|v| v.source());

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
                    hit_by.add_hit(damage_hit.damage(), impact.clone(), unit_source, t);
                }
            }

            // For splash, we need to do some more work, namely figure out what is in range to get
            // damage, and also avoid counting multiple bodies we could hit several times if they
            // are part of the same unit. Because it would be weird if there's two collision boxes
            // next to each other on the same vehicle, and hitting that side causes twice the damage
            // while the other side has one collision box and damage only counts once.
            if let Some(damage_splash) = damage_splash {
                let mut splashes = vec![];

                // We only have to care about hitboxes and hit collections for now.
                let hit_box_with_pose = {
                    let hitboxes = world.component_iter::<HitBox>();
                    hitboxes
                        .map(|(entity, hitbox)| {
                            let pose = world_pose(world, entity);
                            (entity, pose, *hitbox)
                        })
                        .collect::<Vec<_>>()
                };

                for (hitbox_entity, hitbox_pose, hitbox) in hit_box_with_pose.iter() {
                    // Express the impact pose into the hitbox frame.
                    let point_in_hitbox_frame = hitbox_pose.transform().to_inv_h() * impact_pose;
                    let b = AxisAlignedBox::new(hitbox.length(), hitbox.width(), hitbox.height());

                    // Clamp the impact pose to be within the box.
                    let clamped_point = b.clamp_point(point_in_hitbox_frame.to_translation());

                    // Now, the distance between the clamped and point is the distance from the
                    // impact to the nearest point inside the hitbox.
                    let distance =
                        (clamped_point - point_in_hitbox_frame.to_translation()).euclid_norm();

                    if let Some(damage) = damage_splash.damage_by_distance(distance) {
                        splashes.push((*hitbox_entity, damage));
                    }
                }

                let hit_collection_with_pose = {
                    let hitboxes = world.component_iter::<HitCollection>();
                    hitboxes
                        .map(|(entity, hitbox)| {
                            let pose = world_pose(world, entity);
                            (entity, pose, hitbox.clone())
                        })
                        .collect::<Vec<_>>()
                };
                for (hitcollection_entity, hitcollection_pose, hitcollection) in
                    hit_collection_with_pose.iter()
                {
                    let distance = hitcollection
                        .distance_to(**hitcollection_pose, impact_pose.to_translation());
                    if let Some(damage) = damage_splash.damage_by_distance(distance) {
                        splashes.push((*hitcollection_entity, damage));
                    }
                }

                // Ok, now we have all the things that we hit with the splash, we need to deduplicate
                // them by group, and only select the splash that has the highest damage.
                let mut deduplicated_splashes = vec![];
                while !splashes.is_empty() {
                    let last = splashes.pop().unwrap();
                    // If last is part of a group, obtain the group, else just this entry.
                    let group_entities = world
                        .component::<components::group::Group>(last.0)
                        .map(|v| v.entities.to_vec())
                        .unwrap_or_else(|| vec![last.0]);

                    let mut highest_damage_in_group = last.1;
                    // Iterate over the group, and splashes, if entities match, take the max value.
                    for group_member in group_entities.iter() {
                        if let Some(index) = splashes.iter().position(|s| s.0 == *group_member) {
                            highest_damage_in_group =
                                highest_damage_in_group.max(splashes[index].1);
                            splashes.swap_remove(index);
                        }
                    }
                    deduplicated_splashes.push((group_entities[0], highest_damage_in_group));
                }

                // Finally, add a hitby record.

                for (hit_splash_entity, hit_splash_damage) in deduplicated_splashes {
                    // Add HitBy or retrieve.
                    if world
                        .component_mut::<components::hit_by::HitBy>(hit_splash_entity)
                        .is_none()
                    {
                        world.add_component(hit_splash_entity, components::hit_by::HitBy::new());
                    }
                    // Now, we can add a record to the HitBy component.
                    let mut hit_by = world
                        .component_mut::<components::hit_by::HitBy>(hit_splash_entity)
                        .unwrap();
                    hit_by.add_hit(hit_splash_damage, impact.clone(), unit_source, t);
                }
            }
        }

        // Remove all entities;
        world.remove_entities(&impacts);
    }
}
