use battleground_vehicle_control::{Interface, VehicleControl};
use std::f32::consts::PI;

use crate::units::tank;

pub struct TankSwivelShoot {}
impl VehicleControl for TankSwivelShoot {
    fn update(&mut self, interface: &mut dyn Interface) {
        let revolve_pos = 0;
        let revolve_vel = 1;
        let revolve_cmd_vel = 4;

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

        let write_res = interface.set_i32(tank::CANNON_MODULE, 0, true as i32);
        write_res.unwrap();

        let clock = interface.get_f32(tank::CLOCK_MODULE, 0).unwrap();
        if clock < 0.1 {
            interface
                .set_f32(tank::TURRET_MODULE, revolve_cmd_vel, 0.3)
                .unwrap();
            interface
                .set_f32(tank::BARREL_MODULE, revolve_cmd_vel, -0.1)
                .unwrap();
            return;
        }

        // println!("Clock: {clock}");
        // base
        // interface.set_f32(0x1000, 2, 1.0).unwrap();
        // interface.set_f32(0x1000, 3, 1.0).unwrap();

        let turret_pos = interface.get_f32(tank::TURRET_MODULE, revolve_pos).unwrap();
        // println!("turret_pos: {turret_pos}");
        if (turret_pos > PI && turret_pos < (PI * 2.0 - PI / 8.0))
            || (turret_pos > PI / 8.0 && turret_pos < PI)
        {
            interface
                .set_f32(
                    tank::TURRET_MODULE,
                    revolve_cmd_vel,
                    -interface.get_f32(tank::TURRET_MODULE, revolve_vel).unwrap(),
                )
                .unwrap();
        }

        let barrel_pos = interface.get_f32(tank::BARREL_MODULE, revolve_pos).unwrap();
        // println!("barrel_pos: {barrel_pos}");
        if barrel_pos < (PI * 2.0 - PI / 8.0) || (barrel_pos < PI / 8.0) {
            interface
                .set_f32(
                    tank::BARREL_MODULE,
                    revolve_cmd_vel,
                    -interface.get_f32(tank::BARREL_MODULE, revolve_vel).unwrap(),
                )
                .unwrap();
        }

        // interface.set_f32(turret, 4, 1.0).unwrap();
        // interface.set_f32(0x1200, 4, -1.0).unwrap();

        if false {
            let turret_yaw = interface.get_f32(tank::TURRET_MODULE, 0).unwrap();
            let radar_yaw = interface.get_f32(tank::RADAR_ROTATION, 0).unwrap();
            let radar_hits = interface.get_i32(tank::RADAR_MODULE, 0).unwrap();
            for i in 0..radar_hits {
                let offset = i as u32 * 4 + 1;
                let reading_yaw = interface.get_f32(tank::RADAR_MODULE, offset).unwrap();
                let pitch = interface.get_f32(tank::RADAR_MODULE, offset + 1).unwrap();
                let distance = interface.get_f32(tank::RADAR_MODULE, offset + 2).unwrap();
                // let strength = interface.get_f32(tank::RADAR_MODULE, offset + 3).unwrap();
                let combined_yaw =
                    (reading_yaw + radar_yaw + turret_yaw).rem_euclid(std::f32::consts::PI * 2.0);
                let x = combined_yaw.cos() * distance;
                let y = combined_yaw.sin() * distance;
                println!("Radar {i} at {combined_yaw:.2}, {pitch:.2}, x: {x:.3}, y: {y:.3}, dist: {distance:.3}, read yaw: {reading_yaw:?}");
            }
        }
    }
}
