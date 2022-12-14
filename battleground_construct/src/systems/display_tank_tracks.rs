use super::components::differential_drive_base::DifferentialDriveBase;
use super::display::tracks_side::TracksSide;
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

        for (_entity, mut tracks) in world.component_iter_mut::<TracksSide>() {
            // println!("Setting tracks for {_entity:?}");
            let diff_drive_entity = tracks.diff_drive_entity();
            if let Some(base) = world.component::<DifferentialDriveBase>(diff_drive_entity) {
                let (left, right) = base.wheel_velocities();
                // println!("   {left} {right} {dt}");
                tracks.add_track_distance(dt * left, dt * right)
            }
        }
    }
}
