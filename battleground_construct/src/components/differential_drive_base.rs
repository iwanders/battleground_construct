use engine::prelude::*;

#[derive(Copy, Debug, Clone)]
pub struct DifferentialDriveBase {
    pub track_width: f32,
    pub wheel_velocity_bounds: (f32, f32),
    pub wheel_velocity: (f32, f32),
}

impl DifferentialDriveBase {
    pub fn new() -> Self {
        DifferentialDriveBase {
            track_width: 1.0,
            wheel_velocity_bounds: (-1.0, 1.0),
            wheel_velocity: (0.6, 0.8),
        }
    }

    pub fn set_velocities(&mut self, left: f32, right: f32) {
        self.wheel_velocity = (
            left.clamp(self.wheel_velocity_bounds.0, self.wheel_velocity_bounds.1),
            right.clamp(self.wheel_velocity_bounds.0, self.wheel_velocity_bounds.1),
        );
    }

    pub fn track_width(&self) -> f32 {
        self.track_width
    }

    pub fn wheel_velocities(&self) -> (f32, f32) {
        self.wheel_velocity
    }

    pub fn wheel_velocity_bounds(&mut self) -> (f32, f32) {
        self.wheel_velocity_bounds
    }
}
impl Component for DifferentialDriveBase {}
