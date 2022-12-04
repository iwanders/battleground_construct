use battleground_vehicle_control::{Interface, VehicleControl};
use std::f32::consts::PI;

pub struct DummyVehicleControl {}
impl VehicleControl for DummyVehicleControl {
    fn update(&mut self, interface: &mut dyn Interface) {
        let turret = 0x1100;
        let revolve_pos = 0;
        let revolve_vel = 1;
        let revolve_cmd_vel = 4;
        let barrel = 0x1200;
        let cannon = 0x1300;

        interface.set_i32(cannon, 0, true as i32).unwrap();

        if false {
            for m_index in interface.modules().unwrap() {
                println!(
                    "update, module name: {}",
                    interface.module_name(m_index).unwrap()
                );
                for r_index in interface.registers(m_index).unwrap() {
                    println!("  {}", interface.register_name(m_index, r_index).unwrap());
                }
            }
        }

        let clock = interface.get_f32(0x0100, 0).unwrap();
        if clock < 0.1 {
            interface.set_f32(turret, revolve_cmd_vel, 0.3).unwrap();
            interface.set_f32(barrel, revolve_cmd_vel, -0.1).unwrap();
            return;
        }
        // println!("Clock: {clock}");
        // base
        // interface.set_f32(0x1000, 2, 1.0).unwrap();
        // interface.set_f32(0x1000, 3, 1.0).unwrap();

        let turret_pos = interface.get_f32(turret, revolve_pos).unwrap();
        // println!("turret_pos: {turret_pos}");
        if turret_pos > PI && turret_pos < (PI * 2.0 - PI / 8.0) {
            interface
                .set_f32(
                    turret,
                    revolve_cmd_vel,
                    -interface.get_f32(turret, revolve_vel).unwrap(),
                )
                .unwrap();
        } else if turret_pos > PI / 8.0 && turret_pos < PI {
            interface
                .set_f32(
                    turret,
                    revolve_cmd_vel,
                    -interface.get_f32(turret, revolve_vel).unwrap(),
                )
                .unwrap();
        }

        let barrel_pos = interface.get_f32(barrel, revolve_pos).unwrap();
        // println!("barrel_pos: {barrel_pos}");
        if barrel_pos < (PI * 2.0 - PI / 8.0) {
            interface
                .set_f32(
                    barrel,
                    revolve_cmd_vel,
                    -interface.get_f32(barrel, revolve_vel).unwrap(),
                )
                .unwrap();
        } else if barrel_pos < PI / 8.0 {
            interface
                .set_f32(
                    barrel,
                    revolve_cmd_vel,
                    -interface.get_f32(barrel, revolve_vel).unwrap(),
                )
                .unwrap();
        }

        // interface.set_f32(turret, 4, 1.0).unwrap();
        // interface.set_f32(0x1200, 4, -1.0).unwrap();
    }
}

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
        interface.set_f32(0x1000, 2, self.velocities.0).unwrap();
        interface.set_f32(0x1000, 3, self.velocities.1).unwrap();
        interface.set_f32(turret, revolve_cmd_vel, 3.0).unwrap();
    }
}
