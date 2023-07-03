use super::components::revolute::Revolute;
use super::components::tricycle_base::TricycleBase;
use super::components::tricycle_front_wheels::TricycleFrontWheels;
use super::components::tricycle_rear_wheels::TricycleRearWheels;
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
            // Reproduce the velocity calculation here, less than ideal...
            let wheel_vel = base.wheel_velocity();
            let angle = world
                .component::<Revolute>(base.steering_joint())
                .unwrap()
                .position();
            let wheel_base = base.wheel_base();
            let angular_velocity = (wheel_vel / wheel_base) * angle.sin();
            let linear_velocity = wheel_vel * angle.cos();

            if let Some(front_wheels) = world.component::<TricycleFrontWheels>(entity) {
                for wheel_entity in front_wheels.wheels() {
                    if let Some(ref mut wheel) = world.component_mut::<Wheel>(*wheel_entity) {
                        wheel.add_track_distance(dt * wheel_vel)
                    }
                }
            }
            if let Some(rear_wheels) = world.component::<TricycleRearWheels>(entity) {
                let track_width = rear_wheels.track_width();
                for entities in rear_wheels.wheels().chunks(2) {
                    let (left_entity, right_entity) = (entities[0], entities[1]);
                    let left_vel = linear_velocity + track_width / 2.0 * angular_velocity;
                    let right_vel = linear_velocity - track_width / 2.0 * angular_velocity;

                    if let Some(ref mut left_wheel) = world.component_mut::<Wheel>(left_entity) {
                        left_wheel.add_track_distance(dt * left_vel)
                    }
                    if let Some(ref mut right_wheel) = world.component_mut::<Wheel>(right_entity) {
                        right_wheel.add_track_distance(dt * right_vel)
                    }
                }
            }
        }
    }
}
