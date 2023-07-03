use super::components::tricycle_base::TricycleBase;
use super::components::tricycle_front_wheel::TricycleFrontWheel;
use super::display::wheel::Wheel;
use super::Clock;

use engine::prelude::*;

/// System to sync the tricycle wheels with the actuation and effects.
pub struct DisplayTricycleWheels {}
impl System for DisplayTricycleWheels {
    fn update(&mut self, world: &mut World) {
        let (_entity, clock) = world
            .component_iter::<Clock>()
            .next()
            .expect("Should have one clock");
        let dt = clock.step_as_f32();

        for (entity, base) in world.component_iter::<TricycleBase>() {
            // println!("Setting tracks for {_entity:?}");
            let velocity = base.wheel_velocity();
            if let Some(front_wheels) = world.component::<TricycleFrontWheel>(entity) {
                for wheel_entity in front_wheels.wheels() {
                    if let Some(ref mut wheel) = world.component_mut::<Wheel>(*wheel_entity) {
                        wheel.add_track_distance(dt * velocity)
                    }
                }
            }
        }
    }
}
