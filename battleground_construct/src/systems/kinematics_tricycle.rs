use super::components;
use super::components::revolute::Revolute;
use super::components::tricycle_base::TricycleBase;
use super::components::velocity::Velocity;

use engine::prelude::*;

pub struct KinematicsTricycle {}
impl System for KinematicsTricycle {
    fn update(&mut self, world: &mut World) {
        let (_entity, clock) = world
            .component_iter::<components::clock::Clock>()
            .next()
            .expect("Should have one clock");
        let dt = clock.step_as_f32();

        for (entity, ref mut base) in world.component_iter_mut::<TricycleBase>() {
            // Obtain the steering angle
            let angle = world
                .component::<Revolute>(base.steering_joint())
                .unwrap()
                .position();
            let wheel_base = base.wheel_base();

            // First, apply the acceleration of the tricycle drive.
            base.update(dt);

            let wheel_vel = base.wheel_velocity();
            // try to see if we can find a velocity for this entity.
            if let Some(mut vel) = world.component_mut::<Velocity>(entity) {
                let angular_velocity = (wheel_vel / wheel_base) * angle.sin();
                let linear_velocity = wheel_vel * angle.cos();
                *vel = Velocity::from_se2(linear_velocity, 0.0, angular_velocity);
            }
        }
    }
}
