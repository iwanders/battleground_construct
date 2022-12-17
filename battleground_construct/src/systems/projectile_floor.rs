use super::components::point_projectile::PointProjectile;
use super::components::pose::Pose;
use engine::prelude::*;

pub struct ProjectileFloor {}
impl System for ProjectileFloor {
    fn update(&mut self, world: &mut World) {
        for entity in world.component_entities::<PointProjectile>() {
            let below_floor = {
                if let Some(pose) = world.component_mut::<Pose>(entity) {
                    // Yes, so now integrate it.
                    pose.transform().w[2] <= 0.0
                } else {
                    false
                }
            };
            if below_floor {
                {
                    let particles_to_add = world
                        .component::<super::display::particle_emitter::ParticleEmitter>(entity)
                        .map(|p| *p);

                    if let Some(mut copied_particle) = particles_to_add {
                        copied_particle.emitting = false;
                        let impact = world.add_entity();
                        world.add_component(
                            impact,
                            super::components::expiry::Expiry::lifetime(5.0),
                        );
                        world.add_component(impact, copied_particle);
                    }
                }
                world.remove_entity(entity);
            }
        }
    }
}
