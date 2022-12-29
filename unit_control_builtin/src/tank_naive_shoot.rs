use crate::UnitControlResult;
use battleground_unit_control::{Interface, UnitControl};
use std::f32::consts::PI;

use battleground_unit_control::modules::cannon::*;
use battleground_unit_control::modules::clock::*;
use battleground_unit_control::modules::gps::*;
use battleground_unit_control::modules::radar::*;
use battleground_unit_control::modules::radio_receiver::*;
use battleground_unit_control::modules::revolute::*;
use battleground_unit_control::units::tank;

use super::diff_drive_util::angle_diff;

pub struct TankNaiveShoot {
    shoot_at: Option<(f32, f32)>,
    last_seen: f32,
}

impl TankNaiveShoot {
    pub fn new() -> Self {
        TankNaiveShoot {
            shoot_at: None,
            last_seen: -1000.0,
        }
    }
}

impl UnitControl for TankNaiveShoot {
    fn update(&mut self, interface: &mut dyn Interface) -> UnitControlResult {
        // Check the radio for broadcasted friendlies.
        // Check the radar, remove any reflections that are broadcasted friendlies.
        // Put the closest target in shoot_at
        // Orient the gun
        // Lay waste to the target.

        let elapsed = interface
            .get_f32(tank::MODULE_TANK_CLOCK, REG_CLOCK_ELAPSED)
            .unwrap();
        let tank_team = interface
            .get_i32(
                tank::MODULE_TANK_TEAM,
                battleground_unit_control::modules::team::REG_TEAM_TEAMID,
            )
            .unwrap() as u32;

        let tank_x = interface.get_f32(tank::MODULE_TANK_GPS, REG_GPS_X).unwrap();
        let tank_y = interface.get_f32(tank::MODULE_TANK_GPS, REG_GPS_Y).unwrap();
        let tank_z = interface.get_f32(tank::MODULE_TANK_GPS, REG_GPS_Z).unwrap();
        let tank_yaw = interface
            .get_f32(tank::MODULE_TANK_GPS, REG_GPS_YAW)
            .unwrap();

        // Check the radio for broadcasted friendlies.
        let msg_count = interface
            .get_i32(tank::MODULE_TANK_RADIO_RECEIVER, REG_RADIO_RX_MSG_COUNT)
            .unwrap();

        let mut team_xy = vec![];
        for i in 0..msg_count as u32 {
            let msg_offset = REG_RADIO_RX_MSG_START + i * REG_RADIO_RX_MSG_STRIDE;
            let data_offset = msg_offset + REG_RADIO_RX_MSG_OFFSET_DATA;
            let c = interface
                .get_bytes_len(tank::MODULE_TANK_RADIO_RECEIVER, data_offset)
                .unwrap();
            if c != 12 {
                continue;
            }

            let mut d = vec![0; c];
            interface
                .get_bytes(tank::MODULE_TANK_RADIO_RECEIVER, data_offset, &mut d)
                .unwrap();
            // Now that we have the bytes, we can reconstruct the (team, x, y).
            let team = u32::from_le_bytes([d[0], d[1], d[2], d[3]]);
            let x = f32::from_le_bytes([d[4], d[5], d[6], d[7]]);
            let y = f32::from_le_bytes([d[8], d[9], d[10], d[11]]);
            team_xy.push((team, x, y));
        }
        // Drop all messages now that we have obtained them.
        interface
            .set_i32(tank::MODULE_TANK_RADIO_RECEIVER, REG_RADIO_RX_MSG_COUNT, 0)
            .unwrap();

        // Next, check that radar, calculating expressing things in global pose.
        let turret_pos = interface
            .get_f32(tank::MODULE_TANK_REVOLUTE_TURRET, REG_REVOLUTE_POSITION)
            .unwrap();
        let radar_pos = interface
            .get_f32(tank::MODULE_TANK_REVOLUTE_RADAR, REG_REVOLUTE_POSITION)
            .unwrap();
        let radar_yaw = turret_pos + radar_pos + tank_yaw;

        let reflection_count = interface
            .get_i32(tank::MODULE_TANK_RADAR, REG_RADAR_REFLECTION_COUNT)
            .unwrap();

        let mut reflections = vec![];
        for i in 0..reflection_count as u32 {
            let reflection_offset = REG_RADAR_REFLECTION_START + i * REG_RADAR_REFLECTION_STRIDE;
            let yaw_offset = reflection_offset + REG_RADAR_REFLECTION_OFFSET_YAW;
            let distance_offset = reflection_offset + REG_RADAR_REFLECTION_OFFSET_DISTANCE;
            let reflection_yaw = interface
                .get_f32(tank::MODULE_TANK_RADAR, yaw_offset)
                .unwrap();
            let distance = interface
                .get_f32(tank::MODULE_TANK_RADAR, distance_offset)
                .unwrap();
            reflections.push((radar_yaw + reflection_yaw, distance));
        }

        fn distance(p0: (f32, f32), p1: (f32, f32)) -> f32 {
            let dx = p0.0 - p1.0;
            let dy = p0.1 - p0.1;
            (dx * dx + dy * dy).sqrt()
        }

        // Convert those polar coordinates to xy.
        let reflections_xy: Vec<(f32, f32)> = reflections
            .iter()
            .map(|(yaw, distance)| (tank_x + yaw.cos() * distance, tank_y + yaw.sin() * distance))
            .collect();

        let mut enemies = vec![];
        for (ref_x, ref_y) in reflections_xy {
            let mut is_friendly = false;
            for (radio_team, radio_x, radio_y) in team_xy.iter() {
                if *radio_team == tank_team {
                    // check if difference is small.
                    if distance((ref_x, ref_y), (*radio_x, *radio_y)) < 1.0 {
                        is_friendly = true;
                        break;
                    }
                }
            }
            if !is_friendly {
                enemies.push((ref_x, ref_y));
            }
        }

        // Check if any of the enemies is our current target.
        if let Some(shooting_at) = self.shoot_at {
            for p in enemies.iter() {
                if distance(shooting_at, *p) < 1.0 {
                    // Also update the position
                    self.shoot_at = Some(shooting_at);
                    self.last_seen = elapsed;
                    break;
                }
            }
        }
        // Target is not seen for 5 seconds... must be gone.
        if (elapsed - self.last_seen) > 5.0 {
            self.shoot_at = None;
            interface
                .set_i32(tank::MODULE_TANK_CANNON, REG_CANNON_FIRING, false as i32)
                .unwrap();
        }

        // Assign a target.
        if self.shoot_at.is_none() {
            if !enemies.is_empty() {
                self.shoot_at = Some(enemies[0]);
            }
        }

        // Calculate firing solution.
        if let Some(shoot_at) = self.shoot_at {
            // Tackle yaw first.
            let current_yaw = turret_pos + tank_yaw;
            let dx = shoot_at.0 - tank_x;
            let dy = shoot_at.1 - tank_y;
            let desired_yaw = dy.atan2(dx) + PI * 2.0;
            let yaw_error = angle_diff(desired_yaw, current_yaw);
            let min_value = if yaw_error.abs() > 0.1 {
                1.0
            } else {
                yaw_error.abs() * 4.0
            };

            let yaw_error_minned = if yaw_error < 0.0 {
                -yaw_error.max(min_value)
            } else {
                yaw_error.max(min_value)
            };
            interface
                .set_f32(
                    tank::MODULE_TANK_REVOLUTE_TURRET,
                    REG_REVOLUTE_VELOCITY_CMD,
                    yaw_error_minned,
                )
                .unwrap();

            // Then, calculate the angle we need to fire at.
            let distance = (dx * dx + dy * dy).sqrt();
            let barrel_angle = interface
                .get_f32(tank::MODULE_TANK_REVOLUTE_BARREL, REG_REVOLUTE_POSITION)
                .unwrap();
            let barrel_length = 1.0;

            // S is projectile speed. G gravity
            //
            // Z = (S^2 +-  sqrt(S^4 - G(Gx^2 + 2 S^2 y)))/Gx;
            // theta = tan-1(Z)
            let v = 10.0;
            let g = 9.81;
            let turret_rot_to_barrel = 0.25;
            let tank_z_to_barrel_z = 0.375 + 0.1 / 2.0 - 0.25;
            let x = distance - barrel_angle.cos() * barrel_length - turret_rot_to_barrel;
            let y = -tank_z + barrel_angle.sin() * barrel_length - tank_z_to_barrel_z;
            // println!("y: {y}");
            let d = v * v * v * v - g * (g * x * x + 2.0 * v * v * y);

            let s1 = ((v * v + d.sqrt()) / (g * x)).atan();
            let s2 = ((v * v - d.sqrt()) / (g * x)).atan();
            // println!("s1: {s1}, s2: {s2}");
            // Pick the S that's below PI / 4.0
            let angle = s1.clamp(0.0, PI / 2.0).min(s2.clamp(0.0, PI / 2.0));

            if d > 0.0 && angle < (PI / 2.0 + 0.1) {
                let angle_target = 2.0 * PI - angle;
                let angle_error = angle_diff(angle_target, barrel_angle);
                // println!("Calculated {angle:?}, rotating to {angle_target}, current {barrel_angle}, angle_error");

                let min_value = if angle_error.abs() > 0.1 {
                    1.0
                } else {
                    angle_error.abs() * 4.0
                };

                let angle_error_minned = if angle_error < 0.0 {
                    -angle_error.max(min_value)
                } else {
                    angle_error.max(min_value)
                };
                interface
                    .set_f32(
                        tank::MODULE_TANK_REVOLUTE_BARREL,
                        REG_REVOLUTE_VELOCITY_CMD,
                        angle_error_minned,
                    )
                    .unwrap();

                interface
                    .set_i32(tank::MODULE_TANK_CANNON, REG_CANNON_FIRING, true as i32)
                    .unwrap();
            }
        }
        Ok(())
    }
}
