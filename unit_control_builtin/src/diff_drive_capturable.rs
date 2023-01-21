use crate::UnitControlResult;
use battleground_unit_control::modules::objectives::*;
use battleground_unit_control::modules::team::*;
use battleground_unit_control::units::common;
use battleground_unit_control::{Interface, UnitControl};

use crate::diff_drive_util;

pub struct DiffDriveCapturable {}

impl UnitControl for DiffDriveCapturable {
    fn update(&mut self, interface: &mut dyn Interface) -> UnitControlResult {
        // Determine where a capturable is.
        // While we are not there, drive there.
        let team = interface
            .get_i32(common::MODULE_TEAM, REG_TEAM_TEAMID)?;

        let count = interface
            .get_i32(
                common::MODULE_OBJECTIVES,
                REG_OBJECTIVES_CAPTURE_POINT_COUNT,
            )?;
        for i in 0..count as u32 {
            let x = interface
                .get_f32(
                    common::MODULE_OBJECTIVES,
                    REG_OBJECTIVES_CAPTURE_POINT_START + (i * REG_OBJECTIVES_CAPTURE_POINT_STRIDE) + REG_OBJECTIVES_CAPTURE_POINT_OFFSET_X,
                )?;
            let y = interface
                .get_f32(
                    common::MODULE_OBJECTIVES,
                    REG_OBJECTIVES_CAPTURE_POINT_START + (i * REG_OBJECTIVES_CAPTURE_POINT_STRIDE) + REG_OBJECTIVES_CAPTURE_POINT_OFFSET_Y,
                )?;
            let owner = interface
                .get_i32(
                    common::MODULE_OBJECTIVES,
                    REG_OBJECTIVES_CAPTURE_POINT_START + (i * REG_OBJECTIVES_CAPTURE_POINT_STRIDE) + REG_OBJECTIVES_CAPTURE_POINT_OFFSET_OWNER,
                )?;
            let radius = interface
                .get_f32(
                    common::MODULE_OBJECTIVES,
                    REG_OBJECTIVES_CAPTURE_POINT_START + (i * REG_OBJECTIVES_CAPTURE_POINT_STRIDE) + REG_OBJECTIVES_CAPTURE_POINT_OFFSET_RADIUS,
                )?;
            // We don't own this point, lets go there.
            if owner != team {
                diff_drive_util::drive_to_goal((x, y, None), interface, radius)?;
                return Ok(());
            }
        }
        diff_drive_util::stop(interface)?;
        Ok(())
    }
}
