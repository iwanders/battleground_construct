use super::components::hit_box::HitBox;
// use super::components::hit_by::HitBy;
use super::components::hit_plane::HitPlane;
use super::components::hit_sphere::HitSphere;
use super::components::impact::Impact;
use super::components::point_projectile::PointProjectile;
use super::components::pose::world_pose;
use super::components::pose::Pose;
use crate::components::acceleration::Acceleration;
use crate::components::velocity::Velocity;
use crate::display;
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
            .component_iter::<PointProjectile>()
            .map(|(entity, projectile)| (entity, projectile.source()))
            .collect::<Vec<(EntityId, EntityId)>>();

        let projectile_poses = projectiles
            .drain(..)
            .map(|(projectile_id, source_id)| {
                let pose = world_pose(world, projectile_id);
                (projectile_id, source_id, pose)
            })
            .collect::<Vec<(EntityId, EntityId, Pose)>>();

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
            // Remove the point projectile.
            world.remove_component::<PointProjectile>(v.projectile);
            // Remove any velocity of accelartion, fixing the entity in place.
            world.remove_component::<Velocity>(v.projectile);
            world.remove_component::<Acceleration>(v.projectile);

            // Create a bullet destructor.
            let projectile_destructor = world.add_entity();
            let mut destructor =
                crate::display::deconstructor::Deconstructor::new(projectile_destructor);
            destructor.add_impact(v.impact.position(), 0.005);
            destructor.add_element::<crate::display::tank_bullet::TankBullet>(v.projectile, world);
            world.add_component(projectile_destructor, destructor);
            world.add_component(
                projectile_destructor,
                crate::components::expiry::Expiry::lifetime(10.0),
            );

            // Now, we can remove the displayable mesh.
            world.remove_component::<display::tank_bullet::TankBullet>(v.projectile);

            // Add the impact to the entity.
            world.add_component(v.projectile, v.impact);

            // Copy the bullet trail.
            let emitter_id = world.add_entity();
            let emitter = world
                .remove_component::<super::display::particle_emitter::ParticleEmitter>(
                    v.projectile,
                );
            // Disable the particle emitter.
            if let Some(mut emitter) = emitter {
                emitter.emitting = false;
                world.add_component_boxed(emitter_id, emitter);
            }

            world.add_component(emitter_id, super::components::expiry::Expiry::lifetime(5.0));
        }
    }
}
