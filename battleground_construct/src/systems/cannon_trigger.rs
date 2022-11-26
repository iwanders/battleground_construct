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

            // Get the pose of the cannon in the world coordinates. Then create the pose with the
            // Orientation in the global frame.
            let projectile_id = world.add_entity();
            world.add_component::<PointProjectile>(
                &projectile_id,
                PointProjectile::new(cannon_entity.clone()),
            );
            world.add_component::<Pose>(
                &projectile_id,
                Pose::from_mat4(cgmath::Matrix4::<f32>::from_translation(
                    muzzle_pose.clone().w.truncate(),
                )),
            );

            // Calculate the velocity vector in the global frame.
            let mut muzzle_pose = muzzle_pose.transform().clone();
            // zero out the translation components.
            muzzle_pose.w[0] = 0.0;
            muzzle_pose.w[1] = 0.0;
            let v = muzzle_pose * cgmath::Vector4::<f32>::new(muzzle_velocity, 0.0, 0.0, 1.0);
            let projectile_velocity =
                Velocity::from_velocities(v.truncate(), cgmath::Vector3::<f32>::new(0.0, 0.0, 0.0));

            // And add the velocity to the projectile.
            world.add_component::<Velocity>(&projectile_id, projectile_velocity);
            // world.add_component(&projectile_id, crate::display::debug_box::DebugBox::from_size(0.2));
            world.add_component(
                &projectile_id,
                crate::display::tank_bullet::TankBullet::new(),
            );

            // Clearly not the place for this to be... but works for now.
            world.add_component(
                &projectile_id,
                super::components::acceleration::Acceleration::gravity(),
            );

            world.add_component(
                &projectile_id,
                super::display::particle_emitter::ParticleEmitter::from_scale_color(
                    projectile_id,
                    0.05,
                    super::display::Color::MAGENTA,
                ),
            );
        }
    }
}
