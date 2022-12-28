use battleground_unit_control::modules::differential_drive::*;
use battleground_unit_control::modules::gps::*;
use battleground_unit_control::units::tank;
use battleground_unit_control::Interface;

pub fn angle_diff(a: f32, b: f32) -> f32 {
    let a = a - b;
    (a + std::f32::consts::PI).rem_euclid(std::f32::consts::PI * 2.0) - std::f32::consts::PI
}

pub fn drive_to_goal(goal: (f32, f32, f32), interface: &mut dyn Interface) {
    // Get the current position.
    let x = interface.get_f32(tank::MODULE_GPS, REG_GPS_X).unwrap();
    let y = interface.get_f32(tank::MODULE_GPS, REG_GPS_Y).unwrap();
    let yaw = interface.get_f32(tank::MODULE_GPS, REG_GPS_YAW).unwrap();

    let goal_x = goal.0;
    let goal_y = goal.1;
    let _goal_yaw = goal.2;

    let dx = goal_x - x;
    let dy = goal_y - y;

    let desired_orient = dy.atan2(dx);

    let mut left = 0.0;
    let mut right = 0.0;

    let yaw_error = angle_diff(desired_orient, yaw);
    // println!("goal:    {goal_x}, {goal_y}, {goal_yaw}");
    // println!("current: {x}, {y}, {yaw}");
    // println!("yaw_error: {yaw_error}");
    // First, fix the orientation
    if yaw_error.abs() > 0.1 {
        let yaw_error_minned = if yaw_error < 0.0 {
            -yaw_error.max(0.1)
        } else {
            yaw_error.max(0.1)
        };
        left = -yaw_error_minned;
        right = yaw_error_minned;
    } else {
        // We're going in the right direction... full steam ahead if we're not there yet.
        if (dx * dx + dy * dy) > 0.1 {
            left = 10.0;
            right = 10.0;
        }
    }

    interface
        .set_f32(tank::MODULE_DIFF_DRIVE, REG_DIFF_DRIVE_LEFT_CMD, left)
        .unwrap();
    interface
        .set_f32(tank::MODULE_DIFF_DRIVE, REG_DIFF_DRIVE_RIGHT_CMD, right)
        .unwrap();
}

pub fn stop(interface: &mut dyn Interface) {
    interface
        .set_f32(tank::MODULE_DIFF_DRIVE, REG_DIFF_DRIVE_LEFT_CMD, 0.0)
        .unwrap();
    interface
        .set_f32(tank::MODULE_DIFF_DRIVE, REG_DIFF_DRIVE_RIGHT_CMD, 0.0)
        .unwrap();
}
