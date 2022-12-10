use battleground_vehicle_control::{Interface, VehicleControl};

use crate::vehicles::tank;

pub struct DiffDriveForwardsBackwardsControl {
    pub velocities: (f32, f32),
    pub last_flip: f32,
    pub duration: f32,
}

impl VehicleControl for DiffDriveForwardsBackwardsControl {
    fn update(&mut self, interface: &mut dyn Interface) {
        let turret = 0x1100;
        let revolve_cmd_vel = 4;
        let clock = interface.get_f32(0x0100, 0).unwrap();
        if (self.last_flip + self.duration) < clock {
            self.last_flip = clock;
            self.velocities = (self.velocities.0 * -1.0, self.velocities.1 * -1.0);
        }
        // base
        interface
            .set_f32(tank::BASE_MODULE, 2, self.velocities.0)
            .unwrap();
        interface
            .set_f32(tank::BASE_MODULE, 3, self.velocities.1)
            .unwrap();
        interface.set_f32(turret, revolve_cmd_vel, 3.0).unwrap();
    }
}
