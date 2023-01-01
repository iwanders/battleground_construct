use super::components;
use super::components::differential_drive_base::DifferentialDriveBase;
use super::components::velocity::Velocity;

use engine::prelude::*;

pub struct KinematicsDifferentialDrive {}
impl System for KinematicsDifferentialDrive {
    fn update(&mut self, world: &mut World) {
        let (_entity, clock) = world
            .component_iter::<components::clock::Clock>()
            .next()
            .expect("Should have one clock");
        let dt = clock.step_as_f32();
        for (entity, ref mut base) in world.component_iter_mut::<DifferentialDriveBase>() {
            // First, apply the acceleration of the diff drive.
            base.update(dt);
            // try to see if we can find a velocity for this entity.
            if let Some(mut vel) = world.component_mut::<Velocity>(entity) {
                // Yes, so set the velocity.
                let wheel_velocities = base.wheel_velocities();
                let track_width = base.track_width();
                let linear_velocity = (wheel_velocities.0 + wheel_velocities.1) / 2.0;
                let angular_velocity = (wheel_velocities.1 - wheel_velocities.0) / track_width;
                *vel = Velocity::from_se2(linear_velocity, 0.0, angular_velocity);
            }
        }
    }
}
