use super::components::differential_drive_base::DifferentialDriveBase;
use super::display::tank_tracks::TankTracks;
use super::Clock;

use engine::prelude::*;

/// System to sync the tank tracks with the differential drive base that actuates it.
pub struct DisplayTankTracks {}
impl System for DisplayTankTracks {
    fn update(&mut self, world: &mut World) {
        let (_entity, clock) = world
            .component_iter::<Clock>()
            .next()
            .expect("Should have one clock");
        let dt = clock.step_as_f32();

        for (entity, base) in world.component_iter::<DifferentialDriveBase>() {
            if let Some(mut tracks) = world.component_mut::<TankTracks>(entity) {
                let (left, right) = base.wheel_velocities();
                tracks.add_track_distance(dt * left, dt * right)
            }
        }
    }
}
