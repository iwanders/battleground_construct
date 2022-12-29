use crate::UnitControlResult;
use battleground_unit_control::modules::objectives::*;
use battleground_unit_control::modules::team::*;
use battleground_unit_control::units::tank;
use battleground_unit_control::{Interface, UnitControl};

use crate::diff_drive_util;

pub struct DiffDriveCapturable {}

impl UnitControl for DiffDriveCapturable {
    fn update(&mut self, interface: &mut dyn Interface) -> UnitControlResult {
        // Determine where a capturable is.
        // While we are not there, drive there.
        let team = interface
            .get_i32(tank::MODULE_TANK_TEAM, REG_TEAM_TEAMID)
            .unwrap();

        let count = interface
            .get_i32(
                tank::MODULE_TANK_OBJECTIVES,
                REG_OBJECTIVES_CAPTURE_POINT_COUNT,
            )
            .unwrap();
        for i in 0..count as u32 {
            let x = interface
                .get_f32(
                    tank::MODULE_TANK_OBJECTIVES,
                    REG_OBJECTIVES_CAPTURE_POINT_COUNT + 1 + (i * 4) + 0,
                )
                .unwrap();
            let y = interface
                .get_f32(
                    tank::MODULE_TANK_OBJECTIVES,
                    REG_OBJECTIVES_CAPTURE_POINT_COUNT + 1 + (i * 4) + 1,
                )
                .unwrap();
            let owner = interface
                .get_i32(
                    tank::MODULE_TANK_OBJECTIVES,
                    REG_OBJECTIVES_CAPTURE_POINT_COUNT + 1 + (i * 4) + 2,
                )
                .unwrap();

            // We don't own this point, lets go there.
            if owner != team {
                diff_drive_util::drive_to_goal((x, y, 0.0), interface);
                return Ok(());
            }
        }
        diff_drive_util::stop(interface);
        Ok(())
    }
}
