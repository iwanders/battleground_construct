use crate::display::primitives::*;
use battleground_vehicle_control::{Interface, VehicleControl};

use crate::display::draw_module::LineSegment;
use cgmath::vec3;

use crate::vehicles::tank;

pub struct RadarDrawControl {}

impl VehicleControl for RadarDrawControl {
    fn update(&mut self, interface: &mut dyn Interface) {
        let x = interface.get_f32(tank::GPS_MODULE, 0).unwrap();
        let y = interface.get_f32(tank::GPS_MODULE, 1).unwrap();
        let z = interface.get_f32(tank::GPS_MODULE, 2).unwrap();
        let yaw = interface.get_f32(tank::GPS_MODULE, 5).unwrap();

        let turret_pos = interface.get_f32(tank::TURRET_MODULE, 0).unwrap();
        let radar_pos = interface.get_f32(tank::RADAR_ROTATION, 0).unwrap();

        let body_z = 0.25;
        let turret_z = 0.375 + 0.1 / 2.0;
        let radar_z = 0.07;

        // radar position in world:
        let rot = yaw + turret_pos + radar_pos;
        let local_rotation = Mat4::from_angle_z(cgmath::Rad(rot));
        let local_offset = Mat4::from_translation(vec3(0.0, 0.0, body_z + radar_z + turret_z));
        let global_offset =
            Mat4::from_translation(vec3(x, y, z)) * Mat4::from_angle_z(cgmath::Rad(yaw));

        let global_radar = global_offset * local_offset * local_rotation;

        let mut lines = vec![];

        lines.push(LineSegment {
            p0: [0.0, 0.0, 0.0],
            p1: [0.0, 0.0, 5.0],
            width: 0.05,
            color: Color::WHITE,
        });

        let radar_hits = interface.get_i32(tank::RADAR_MODULE, 0).unwrap();
        for i in 0..radar_hits {
            let offset = i as u32 * 4 + 1;
            let reading_yaw = interface.get_f32(tank::RADAR_MODULE, offset + 0).unwrap();
            let pitch = interface.get_f32(tank::RADAR_MODULE, offset + 1).unwrap();
            let distance = interface.get_f32(tank::RADAR_MODULE, offset + 2).unwrap();
            // let strength = interface.get_f32(tank::RADAR_MODULE, offset + 3).unwrap();
            // let combined_yaw =
            // (reading_yaw + radar_yaw + turret_yaw).rem_euclid(std::f32::consts::PI * 2.0);
            let target_x = reading_yaw.cos() * distance;
            let target_y = reading_yaw.sin() * distance;
            let target_z = pitch.sin() * distance;
            let local_target = Mat4::from_translation(vec3(target_x, target_y, target_z));
            let global_target = global_radar * local_target;
            println!(
                "Radar {i} at {:?}, we're at {:?}",
                global_target.w.truncate(),
                global_radar.w.truncate()
            );
            lines.push(LineSegment {
                p0: [global_radar.w.x, global_radar.w.y, global_radar.w.z],
                p1: [global_target.w.x + x, global_target.w.y, global_target.w.z],
                width: 0.05,
                color: Color::RED,
            });
        }

        // Now we just need to convert the struct to bytes.
        let mut draw_instructions: Vec<u8> = vec![];
        for l in lines {
            draw_instructions.extend(l.into_le_bytes());
        }
        interface
            .set_bytes(tank::DRAW_MODULE, 0, &draw_instructions)
            .expect("");
    }
}
