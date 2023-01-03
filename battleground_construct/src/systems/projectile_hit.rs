use super::components::hit_box::HitBox;
use super::components::hit_effect::HitEffect;
use super::components::hit_plane::HitPlane;
use super::components::hit_sphere::HitSphere;
use super::components::impact::Impact;
use super::components::point_projectile::PointProjectile;
use super::components::pose::world_pose;
use super::components::pose::Pose;
use crate::components::acceleration::Acceleration;
use crate::components::unit::UnitId;
use crate::components::unit_source::UnitSource;
use crate::components::velocity::Velocity;
use crate::util::box_collision::AxisAlignedBox;
use crate::util::cgmath::prelude::*;
use engine::prelude::*;

pub struct ProjectileHit {}
impl System for ProjectileHit {
    fn update(&mut self, world: &mut World) {
        struct HitState {
            projectile: EntityId,
            impact: Impact,
        }

        // This fails if at any point someone applies a HitSphere to a PointProjectile.
        // Complexity is O(HitSphere * PointProjectile)...
        let mut projectile_hits: Vec<HitState> = vec![];

        // Get all projectiles' world poses.
        let mut projectiles = world
            .component_entities::<PointProjectile>()
            .iter()
            .map(|entity| {
                (
                    *entity,
                    world.component::<UnitSource>(*entity).map(|v| v.source()),
                )
            })
            .collect::<Vec<(EntityId, Option<UnitId>)>>();

        let projectile_poses = projectiles
            .drain(..)
            .map(|(projectile_id, source_id)| {
                let pose = world_pose(world, projectile_id);
                (projectile_id, source_id, pose)
            })
            .collect::<Vec<(EntityId, Option<UnitId>, Pose)>>();

        {
            // Get all the hit planes
            let hit_sphere_with_pose = {
                let hitplanes = world.component_iter::<HitPlane>();
                hitplanes
                    .map(|(entity, sphere)| {
                        let pose = world_pose(world, entity);
                        (entity, pose, sphere)
                    })
                    .collect::<Vec<_>>()
            };
            for (projectile_entity, source_id, projectile_pose) in projectile_poses.iter() {
                for (hitplane_entity, hitplane_pose, hitplane) in hit_sphere_with_pose.iter() {
                    // convert the projectile pose into the hitbox's local frame.
                    // currently, projectile_pose is world -> projectile.
                    //            hitbox_pose is world -> hitbox.
                    let point_in_hitplane_frame =
                        hitplane_pose.transform().to_inv_h() * projectile_pose.transform();
                    let inside = hitplane.above(point_in_hitplane_frame.to_translation());
                    if inside {
                        let v = HitState {
                            projectile: *projectile_entity,
                            impact: Impact::new(
                                Some(*hitplane_entity),
                                *projectile_pose.transform(),
                                *source_id,
                            ),
                        };
                        projectile_hits.push(v);
                    }
                }
            }

            // Get all the hitspheres
            let hit_sphere_with_pose = {
                let hitspheres = world.component_iter::<HitSphere>();
                hitspheres
                    .map(|(entity, sphere)| {
                        let pose = world_pose(world, entity);
                        (entity, pose, sphere)
                    })
                    .collect::<Vec<_>>()
            };

            // And now, we can do the nested for loop.
            for (projectile_entity, source_id, projectile_pose) in projectile_poses.iter() {
                for (sphere_entity, sphere_pose, sphere) in hit_sphere_with_pose.iter() {
                    let diff = projectile_pose.w - sphere_pose.w;
                    let dist = diff.x * diff.x + diff.y * diff.y + diff.z + diff.z;
                    let inside = dist < (sphere.radius() * sphere.radius());
                    if inside {
                        // println!("{projectile_entity:?} is inside of {sphere_entity:?}!");
                        let v = HitState {
                            projectile: *projectile_entity,
                            impact: Impact::new(
                                Some(*sphere_entity),
                                *projectile_pose.transform(),
                                *source_id,
                            ),
                        };
                        projectile_hits.push(v);
                    }
                }
            }

            // Next, get all hitboxes.
            let hit_box_with_pose = {
                let hitboxes = world.component_iter::<HitBox>();
                hitboxes
                    .map(|(entity, hitbox)| {
                        let pose = world_pose(world, entity);
                        (entity, pose, hitbox)
                    })
                    .collect::<Vec<_>>()
            };

            for (projectile_entity, source_id, projectile_pose) in projectile_poses.iter() {
                for (hitbox_entity, hitbox_pose, hitbox) in hit_box_with_pose.iter() {
                    // convert the projectile pose into the hitbox's local frame.
                    // currently, projectile_pose is world -> projectile.
                    //            hitbox_pose is world -> hitbox.
                    // hitbox -> world -> world -> projectile.
                    let point_in_hitbox_frame =
                        hitbox_pose.transform().to_inv_h() * projectile_pose.transform();
                    let b = AxisAlignedBox::new(hitbox.length(), hitbox.width(), hitbox.height());
                    let inside = b.is_inside(point_in_hitbox_frame.to_translation());
                    if inside {
                        let v = HitState {
                            projectile: *projectile_entity,
                            impact: Impact::new(
                                Some(*hitbox_entity),
                                *projectile_pose.transform(),
                                *source_id,
                            ),
                        };
                        projectile_hits.push(v);
                    }
                }
            }
        }

        for v in projectile_hits {
            // Run the hit effect before modifying any projectile components.
            let hit_effect = world
                .component::<HitEffect>(v.projectile)
                .map(|z| z.effect());
            if let Some(effect_fn) = hit_effect {
                effect_fn(world, v.projectile, &v.impact);
            }

            // Remove the point projectile.
            world.remove_component::<PointProjectile>(v.projectile);
            // Remove any velocity of acceleration, fixing the entity in place.
            world.remove_component::<Velocity>(v.projectile);
            world.remove_component::<Acceleration>(v.projectile);

            // Add the impact to the entity.
            world.add_component(v.projectile, v.impact);
        }
    }
}
