use super::components::cannon::Cannon;
use super::components::pose::Pose;
use super::components::velocity::Velocity;
use super::components::point_projectile::PointProjectile;
use super::Clock;
use engine::prelude::*;

pub struct CannonTrigger {}
impl System for CannonTrigger {
    fn update(&mut self, world: &mut World) {
        let current = {let (_entity, clock) = world
            .component_iter_mut::<Clock>()
            .next()
            .expect("Should have one clock");
            clock.elapsed_as_f32()};

        for cannon_entity in world.component_entities::<Cannon>().iter() {
            let (fired, muzzle_velocity) = {
                let mut cannon = world.component_mut::<Cannon>(&cannon_entity).unwrap();
                if cannon.is_ready(current) {
                    cannon.fire(current);
                    (true, cannon.muzzle_velocity())
                } else {
                    (false, 0.0)
                }
            };

            if !fired {
                continue;
            }
            let muzzle_pose = super::components::pose::world_pose(world, &cannon_entity);


            // get the pose of the cannon in the world coordinates.
            // spawn a new projectile.
            let projectile_id = world.add_entity();
            world.add_component::<PointProjectile>(&projectile_id, PointProjectile::new(cannon_entity.clone()));
            world.add_component::<Pose>(&projectile_id, muzzle_pose.clone());

            
            // We added the pose, we can now hack up the muzzle_pose.
            // Velocity is in local frame...
            let v = cgmath::Vector3::<f32>::new(muzzle_velocity, 0.0, 0.0);
            let projectile_velocity = Velocity::from_velocities(v, cgmath::Vector3::<f32>::new(0.0, 0.0, 0.0));
            world.add_component::<Velocity>(&projectile_id, projectile_velocity);
            world.add_component(&projectile_id, crate::display::debug_box::DebugBox::from_size(0.2));
                
            
        }
    }
}
