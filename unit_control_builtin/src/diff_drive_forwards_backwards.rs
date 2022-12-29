use crate::UnitControlResult;
use battleground_unit_control::units::tank;
use battleground_unit_control::{Interface, UnitControl};

pub struct DiffDriveForwardsBackwardsControl {
    pub velocities: (f32, f32),
    pub last_flip: f32,
    pub duration: f32,
}

impl UnitControl for DiffDriveForwardsBackwardsControl {
    fn update(&mut self, interface: &mut dyn Interface) -> UnitControlResult {
        let revolve_cmd_vel = 4;
        let clock = interface.get_f32(tank::MODULE_TANK_CLOCK, 0).unwrap();
        if (self.last_flip + self.duration) < clock {
            self.last_flip = clock;
            self.velocities = (self.velocities.0 * -1.0, self.velocities.1 * -1.0);
        }
        // base
        interface
            .set_f32(tank::MODULE_TANK_DIFF_DRIVE, 2, self.velocities.0)
            .unwrap();
        interface
            .set_f32(tank::MODULE_TANK_DIFF_DRIVE, 3, self.velocities.1)
            .unwrap();
        interface
            .set_f32(tank::MODULE_TANK_REVOLUTE_TURRET, revolve_cmd_vel, 3.0)
            .unwrap();
        // interface.set_f32(tank::MODULE_TANK_REVOLUTE_BARREL, revolve_cmd_vel, 3.0).unwrap();
        Ok(())
    }
}
