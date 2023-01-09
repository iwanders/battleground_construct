use super::components;
use engine::prelude::*;

/// This handles logic necessary at the end of playback.
pub struct PlaybackFinished {}
impl System for PlaybackFinished {
    fn update(&mut self, world: &mut World) {
        let is_finished = world
            .component_iter::<components::recording::PlaybackFinishedMarker>()
            .next()
            .is_some();
        if is_finished {
            // Halt the wheels, otherwise they keep integrating infinitely.
            for (_entity, mut diff) in world
                .component_iter_mut::<components::differential_drive_base::DifferentialDriveBase>(
            ) {
                *diff.wheel_velocities_mut() = (0.0, 0.0);
            }
        }
    }
}
