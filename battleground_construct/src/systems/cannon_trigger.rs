use super::components::cannon::Cannon;
use super::components::pose::Pose;
// use super::components::velocity::GlobalVelocity;
use super::components::point_projectile::PointProjectile;
use super::components::velocity::Velocity;
use super::Clock;
use engine::prelude::*;

pub struct CannonTrigger {}
impl System for CannonTrigger {
    fn update(&mut self, world: &mut World) {
        let current = {
            let (_entity, clock) = world
                .component_iter_mut::<Clock>()
                .next()
                .expect("Should have one clock");
            clock.elapsed_as_f32()
        };

        for cannon_entity in world.component_entities::<Cannon>().iter() {
            let fired = {
                let mut cannon = world.component_mut::<Cannon>(&cannon_entity).unwrap();
                if cannon.is_ready(current) {
                    cannon.fire(current);
                    true
                } else {
                    false
                }
            };

            if !fired {
                continue;
            }

            let muzzle_pose = super::components::pose::world_pose(world, &cannon_entity);

            let cannon_effect = { world.component::<Cannon>(&cannon_entity).unwrap().effect() };

            cannon_effect(world, &muzzle_pose, cannon_entity);
        }
    }
}
