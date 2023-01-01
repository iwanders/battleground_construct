use crate::UnitControlResult;
use battleground_unit_control::{Interface, UnitControl};
// use std::f32::consts::PI;

use battleground_unit_control::modules::cannon::*;
use battleground_unit_control::modules::radar::*;
use battleground_unit_control::modules::revolute::*;
use battleground_unit_control::units::tank;

pub struct TankSwivelShoot {
    init_done: bool,
    turret_swivel_interval: f32,
    turret_flip_last_time: f32,
    barrel_swivel_interval: f32,
    barrel_flip_last_time: f32,
}

impl TankSwivelShoot {
    pub fn new() -> Self {
        TankSwivelShoot {
            init_done: false,
            turret_swivel_interval: 3.0,
            turret_flip_last_time: -1.5, // half of interval, for balanced swivel
            barrel_swivel_interval: 4.0,
            barrel_flip_last_time: 0.0,
        }
    }
}

impl UnitControl for TankSwivelShoot {
    fn update(&mut self, interface: &mut dyn Interface) -> UnitControlResult {
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

        let write_res =
            interface.set_i32(tank::MODULE_TANK_CANNON, REG_CANNON_TRIGGER, true as i32);
        write_res.unwrap();

        let clock = interface.get_f32(tank::MODULE_TANK_CLOCK, 0).unwrap();
        if !self.init_done {
            interface
                .set_f32(
                    tank::MODULE_TANK_REVOLUTE_TURRET,
                    REG_REVOLUTE_VELOCITY_CMD,
                    0.3,
                )
                .unwrap();
            interface
                .set_f32(
                    tank::MODULE_TANK_REVOLUTE_BARREL,
                    REG_REVOLUTE_VELOCITY_CMD,
                    -0.1,
                )
                .unwrap();
            self.init_done = true;
            return Ok(());
        }

        // println!("Clock: {clock}");
        // base
        // interface.set_f32(0x1000, 2, 1.0).unwrap();
        // interface.set_f32(0x1000, 3, 1.0).unwrap();

        // let turret_pos = interface
        // .get_f32(tank::MODULE_TANK_REVOLUTE_TURRET, REG_REVOLUTE_POSITION)
        // .unwrap();
        // println!("turret_pos: {turret_pos}");
        if (clock - self.turret_flip_last_time) > self.turret_swivel_interval {
            interface
                .set_f32(
                    tank::MODULE_TANK_REVOLUTE_TURRET,
                    REG_REVOLUTE_VELOCITY_CMD,
                    -interface
                        .get_f32(tank::MODULE_TANK_REVOLUTE_TURRET, REG_REVOLUTE_VELOCITY)
                        .unwrap(),
                )
                .unwrap();
            self.turret_flip_last_time = clock;
        }

        // let barrel_pos = interface
        // .get_f32(tank::MODULE_TANK_REVOLUTE_BARREL, REG_REVOLUTE_POSITION)
        // .unwrap();
        // println!("barrel_pos: {barrel_pos}");
        if (clock - self.barrel_flip_last_time) > self.barrel_swivel_interval {
            interface
                .set_f32(
                    tank::MODULE_TANK_REVOLUTE_BARREL,
                    REG_REVOLUTE_VELOCITY_CMD,
                    -interface
                        .get_f32(tank::MODULE_TANK_REVOLUTE_BARREL, REG_REVOLUTE_VELOCITY)
                        .unwrap(),
                )
                .unwrap();
            self.barrel_flip_last_time = clock;
        }

        // interface.set_f32(turret, 4, 1.0).unwrap();
        // interface.set_f32(0x1200, 4, -1.0).unwrap();

        if false {
            let turret_yaw = interface
                .get_f32(tank::MODULE_TANK_REVOLUTE_TURRET, REG_REVOLUTE_POSITION)
                .unwrap();
            let radar_yaw = interface
                .get_f32(tank::MODULE_TANK_REVOLUTE_RADAR, REG_REVOLUTE_POSITION)
                .unwrap();
            let radar_hits = interface
                .get_i32(tank::MODULE_TANK_RADAR, REG_RADAR_REFLECTION_COUNT)
                .unwrap();
            for i in 0..radar_hits {
                let offset = i as u32 * REG_RADAR_REFLECTION_STRIDE + REG_RADAR_REFLECTION_START;
                let reading_yaw = interface
                    .get_f32(
                        tank::MODULE_TANK_RADAR,
                        offset + REG_RADAR_REFLECTION_OFFSET_YAW,
                    )
                    .unwrap();
                let pitch = interface
                    .get_f32(
                        tank::MODULE_TANK_RADAR,
                        offset + REG_RADAR_REFLECTION_OFFSET_PITCH,
                    )
                    .unwrap();
                let distance = interface
                    .get_f32(
                        tank::MODULE_TANK_RADAR,
                        offset + REG_RADAR_REFLECTION_OFFSET_DISTANCE,
                    )
                    .unwrap();
                // let strength = interface.get_f32(tank::MODULE_TANK_RADAR, offset + 3).unwrap();
                let combined_yaw =
                    (reading_yaw + radar_yaw + turret_yaw).rem_euclid(std::f32::consts::PI * 2.0);
                let x = combined_yaw.cos() * distance;
                let y = combined_yaw.sin() * distance;
                println!("Radar {i} at {combined_yaw:.2}, {pitch:.2}, x: {x:.3}, y: {y:.3}, dist: {distance:.3}, read yaw: {reading_yaw:?}");
            }
        }
        Ok(())
    }
}
