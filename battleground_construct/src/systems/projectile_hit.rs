use super::components::hit_by::HitBy;
use super::components::hit_sphere::HitSphere;
use super::components::point_projectile::PointProjectile;
use super::components::pose::world_pose;
use super::components::pose::Pose;
use engine::prelude::*;

pub struct ProjectileHit {}
impl System for ProjectileHit {
    fn update(&mut self, world: &mut World) {
        // This fails if at any point someone applies a HitSphere to a PointProjectile.
        // Complexity is O(HitSphere * PointProjectile)...
        let mut hit_projectile_source: Vec<(EntityId, EntityId, EntityId)> = vec![];

        // Get all projectiles' world poses.
        let mut projectiles = world
            .component_iter::<PointProjectile>()
            .map(|(entity, projectile)| (entity, projectile.source()))
            .collect::<Vec<(EntityId, EntityId)>>();
        let projectile_poses = projectiles
            .drain(..)
            .map(|(projectile_id, source_id)| {
                let pose = world_pose(world, &projectile_id);
                (projectile_id, source_id, pose)
            })
            .collect::<Vec<(EntityId, EntityId, Pose)>>();

        {
            // Get all the hitspheres
            let hitspheres = world.component_iter::<HitSphere>();
            let hit_sphere_with_pose = hitspheres
                .map(|(entity, sphere)| {
                    let pose = world_pose(world, &entity);
                    (entity, pose, sphere)
                })
                .collect::<Vec<_>>();

            // And now, we can do the nested for loop.
            for (projectile_entity, source_id, projectile_pose) in projectile_poses.iter() {
                for (sphere_entity, sphere_pose, sphere) in hit_sphere_with_pose.iter() {
                    let diff = projectile_pose.w - sphere_pose.w;
                    let dist = diff.x * diff.x + diff.y * diff.y + diff.z + diff.z;
                    let inside = dist < (sphere.radius() * sphere.radius());
                    if inside {
                        println!("{projectile_entity:?} is inside of {sphere_entity:?}!");
                        hit_projectile_source.push((
                            *sphere_entity,
                            *projectile_entity,
                            *source_id,
                        ));
                    }
                }
            }
        }

        for (hit_sphere_entity, projectile_entity, source_entity) in hit_projectile_source {
            world.add_component(
                &hit_sphere_entity,
                HitBy::new(projectile_entity, source_entity),
            );
        }
    }
}
