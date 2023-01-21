use battleground_unit_control::modules::differential_drive::*;
use battleground_unit_control::modules::gps::*;
use battleground_unit_control::units::common;
use battleground_unit_control::units::tank;
use battleground_unit_control::Interface;

pub fn angle_diff(a: f32, b: f32) -> f32 {
    let a = a - b;
    (a + std::f32::consts::PI).rem_euclid(std::f32::consts::PI * 2.0) - std::f32::consts::PI
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn foo() {
        println!("angle_diff(0.0, 6.20): {}", angle_diff(0.0, 6.2));
        println!("angle_diff(0.0, -6.20): {}", angle_diff(0.0, -6.2));
        println!("angle_diff(6.20, 0.0): {}", angle_diff(6.2, 0.0));
        println!("angle_diff(-6.20, 0.0): {}", angle_diff(-6.2, 0.0));
        println!("angle_diff(0.1, 0.0): {}", angle_diff(0.1, 0.0));
        println!("angle_diff(-0.1, 0.0): {}", angle_diff(-0.1, 0.0));
        println!("angle_diff(-0.1, -0.1): {}", angle_diff(-0.1, -0.1));
    }
}

pub fn drive_to_goal(goal: (f32, f32, Option<f32>), interface: &mut dyn Interface, tolerance: f32) -> Result<(), Box<dyn std::error::Error>>{
    // Get the current position.
    let x = interface.get_f32(common::MODULE_GPS, REG_GPS_X)?;
    let y = interface.get_f32(common::MODULE_GPS, REG_GPS_Y)?;
    let yaw = interface.get_f32(common::MODULE_GPS, REG_GPS_YAW)?;

    let goal_x = goal.0;
    let goal_y = goal.1;
    // let goal_yaw = goal.2;

    // let goal_x = -3.0;
    // let goal_y = -3.0;
    // let goal_yaw = None;

    let dx = goal_x - x;
    let dy = goal_y - y;

    let desired_orient = dy.atan2(dx);

    let yaw_error = angle_diff(desired_orient, yaw);
    let distance = (dx * dx + dy * dy).sqrt();
    if distance < tolerance {
        return stop(interface);
    }
    let yaw_p = 0.5;

    // make steering more important if there's yaw error.
    let left_steer = -yaw_error * yaw_p;
    let right_steer = yaw_error * yaw_p;

    let only_steer_yaw_error = 0.25;
    let steer_r = if yaw_error.abs() > only_steer_yaw_error {
        1.0
    } else {
        0.5
    };

    let left_forward = distance.clamp(0.0, 1.0);
    let right_forward = distance.clamp(0.0, 1.0);

    let left = left_steer * steer_r + left_forward * (1.0 - steer_r);
    let right = right_steer * steer_r + right_forward * (1.0 - steer_r);

    interface
        .set_f32(tank::MODULE_TANK_DIFF_DRIVE, REG_DIFF_DRIVE_LEFT_CMD, left)?;
    interface
        .set_f32(
            tank::MODULE_TANK_DIFF_DRIVE,
            REG_DIFF_DRIVE_RIGHT_CMD,
            right,
        )?;
    Ok(())
}

pub fn stop(interface: &mut dyn Interface)  -> Result<(), Box<dyn std::error::Error>>{
    interface
        .set_f32(tank::MODULE_TANK_DIFF_DRIVE, REG_DIFF_DRIVE_LEFT_CMD, 0.0)?;
    interface
        .set_f32(tank::MODULE_TANK_DIFF_DRIVE, REG_DIFF_DRIVE_RIGHT_CMD, 0.0)?;
    Ok(())
}