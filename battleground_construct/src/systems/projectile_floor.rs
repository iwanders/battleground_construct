use super::components::point_projectile::PointProjectile;
use super::components::pose::Pose;
use engine::prelude::*;

pub struct ProjectileFloor {}
impl System for ProjectileFloor {
    fn update(&mut self, world: &mut World) {
        for entity in world.component_entities::<PointProjectile>() {
            let below_floor = {
                if let Some(pose) = world.component_mut::<Pose>(&entity) {
                    // Yes, so now integrate it.
                    pose.transform().w[2] <= 0.0
                } else {
                    false
                }
            };
            if below_floor {
                world.remove_entity(&entity);
            }
        }
    }
}
