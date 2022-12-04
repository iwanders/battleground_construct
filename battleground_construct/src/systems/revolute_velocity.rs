use super::components::revolute::Revolute;
use super::components::velocity::Velocity;
use super::Clock;
use engine::prelude::*;

pub struct RevoluteVelocity {}
impl System for RevoluteVelocity {
    fn update(&mut self, world: &mut World) {
        let (_entity, clock) = world
            .component_iter_mut::<Clock>()
            .next()
            .expect("Should have one clock");
        let dt = clock.step_as_f32();
        for (entity, mut rev) in world.component_iter_mut::<Revolute>() {
            rev.integrate(dt);
            if let Some(mut vel) = world.component_mut::<Velocity>(entity) {
                *vel = rev.to_twist().into();
            }
        }
    }
}
