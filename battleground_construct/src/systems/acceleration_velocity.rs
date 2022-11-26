use super::components::acceleration::Acceleration;
use super::components::velocity::Velocity;
use super::Clock;
use engine::prelude::*;

pub struct AccelerationVelocity {}
impl System for AccelerationVelocity {
    fn update(&mut self, world: &mut World) {
        let (_entity, clock) = world
            .component_iter::<Clock>()
            .next()
            .expect("Should have one clock");
        let dt = clock.step_as_f32();

        for (entity, accel) in world.component_iter::<Acceleration>() {
            // try to see if we can find a velocity for this entity.
            if let Some(mut vel) = world.component_mut::<Velocity>(entity) {
                // Yes, so now integrate it.
                *vel = accel.integrate_velocity(&vel, dt);
            }
        }
    }
}
