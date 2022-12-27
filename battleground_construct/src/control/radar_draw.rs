use crate::display::primitives::*;
use battleground_unit_control::{Interface, UnitControl};

use crate::display::draw_module::LineSegment;
// use crate::units::tank;
use crate::util::cgmath::prelude::*;
use cgmath::vec3;

// this is a private test controller.

use battleground_unit_control::units::tank;

pub struct RadarDrawControl {}

const GREEN: [u8; 4] = [0, 255, 0, 255];
const BLUE: [u8; 4] = [0, 0, 255, 255];
const RED: [u8; 4] = [255, 0, 0, 255];
use battleground_unit_control::modules::gps::*;
use battleground_unit_control::modules::radar::*;
use battleground_unit_control::modules::revolute::*;
impl UnitControl for RadarDrawControl {
    fn update(&mut self, interface: &mut dyn Interface) {
        let x = interface.get_f32(tank::MODULE_GPS, REG_X).unwrap();
        let y = interface.get_f32(tank::MODULE_GPS, REG_Y).unwrap();
        let z = interface.get_f32(tank::MODULE_GPS, REG_Z).unwrap();
        let yaw = interface.get_f32(tank::MODULE_GPS, REG_YAW).unwrap();

        let body_z = 0.25;
        let turret_z = 0.375 + 0.1 / 2.0;
        let radar_z = 0.07;

        // Gps is attached to the body.
        // Radar is not located in the gps, instead it has the offset body_z.

        // Calculate the position of the base;
        let global_offset =
            Mat4::from_translation(vec3(x, y, z)) * Mat4::from_angle_z(cgmath::Rad(yaw));
        // We draw in the base frame, so to go from global to draw, we need that inverse.
        let draw_to_global = global_offset.to_inv_h();
        // Then, we can calculate everything in global frame, and finally draw in local.

        let turret_pos = interface
            .get_f32(tank::MODULE_REVOLUTE_TURRET, REG_POSITION)
            .unwrap();
        let radar_pos = interface
            .get_f32(tank::MODULE_REVOLUTE_RADAR, REG_POSITION)
            .unwrap();

        // radar position in world:
        let rot = turret_pos + radar_pos;
        let local_rotation = Mat4::from_angle_z(cgmath::Rad(rot));
        let radar_offset = Mat4::from_translation(vec3(0.0, 0.0, radar_z + turret_z - body_z));
        let local_radar = radar_offset * local_rotation;

        let global_radar = draw_to_global * global_offset * local_radar;

        let mut lines = vec![];

        lines.push(LineSegment {
            p0: [0.0, 0.0, 0.0],
            p1: [0.0, 0.0, 5.0],
            width: 0.01,
            color: GREEN,
        });
        use cgmath::Rad;

        let radar_range_max = interface
            .get_f32(tank::MODULE_RADAR, REG_RANGE_MAX)
            .unwrap();
        let radar_detection_yaw = interface
            .get_f32(tank::MODULE_RADAR, REG_DETECTION_ANGLE_YAW)
            .unwrap();
        let _radar_detection_pitch = interface
            .get_f32(tank::MODULE_RADAR, REG_DETECTION_ANGLE_PITCH)
            .unwrap();

        let p1 = global_offset
            * local_radar
            * Mat4::from_angle_z(Rad(-radar_detection_yaw))
            * vec3(radar_range_max, 0.0, 0.0).to_h();
        let p1 = draw_to_global * p1;

        lines.push(LineSegment {
            p0: [global_radar.w.x, global_radar.w.y, global_radar.w.z],
            p1: [p1.w.x, p1.w.y, p1.w.z],
            width: 0.01,
            color: BLUE,
        });

        let p1 = global_offset
            * local_radar
            * Mat4::from_angle_z(Rad(radar_detection_yaw))
            * vec3(radar_range_max, 0.0, 0.0).to_h();
        let p1 = draw_to_global * p1;

        lines.push(LineSegment {
            p0: [global_radar.w.x, global_radar.w.y, global_radar.w.z],
            p1: [p1.w.x, p1.w.y, p1.w.z],
            width: 0.01,
            color: BLUE,
        });

        let radar_hits = interface
            .get_i32(tank::MODULE_RADAR, REG_REFLECTION_COUNT)
            .unwrap();
        for i in 0..radar_hits {
            let offset = i as u32 * REG_REFLECTION_STRIDE + REG_REFLECTION_START;
            let reading_yaw = interface
                .get_f32(tank::MODULE_RADAR, offset + REG_REFLECTION_OFFSET_YAW)
                .unwrap();
            let pitch = interface
                .get_f32(tank::MODULE_RADAR, offset + REG_REFLECTION_OFFSET_PITCH)
                .unwrap();
            let distance = interface
                .get_f32(tank::MODULE_RADAR, offset + REG_REFLECTION_OFFSET_DISTANCE)
                .unwrap();
            // println!("Reading {i}: yaw: {reading_yaw}, pitch: {pitch}, distance: {distance}");

            let radar_hit_frame = Mat4::from_angle_z(cgmath::Rad(reading_yaw))
                * Mat4::from_angle_y(cgmath::Rad(pitch));
            let radar_hit = radar_hit_frame * vec3(distance, 0.0, 0.0).to_h();
            let target = local_radar * radar_hit;
            let draw_target = target;

            lines.push(LineSegment {
                p0: [global_radar.w.x, global_radar.w.y, global_radar.w.z],
                p1: [draw_target.w.x, draw_target.w.y, draw_target.w.z - body_z],
                width: 0.05,
                color: RED,
            });
        }

        // Now we just need to convert the struct to bytes.
        let mut draw_instructions: Vec<u8> = vec![];
        for l in lines {
            draw_instructions.extend(l.into_le_bytes());
        }
        interface
            .set_bytes(
                tank::MODULE_DRAW,
                battleground_unit_control::modules::draw::REG_LINES,
                &draw_instructions,
            )
            .expect("");
    }
}
