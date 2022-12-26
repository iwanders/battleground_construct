use battleground_unit_control::units::tank;
use battleground_unit_control::Interface;

pub fn drive_to_goal(goal: (f32, f32, f32), interface: &mut dyn Interface) {
    // Get the current position.
    let x = interface.get_f32(tank::GPS_MODULE, 0).unwrap();
    let y = interface.get_f32(tank::GPS_MODULE, 1).unwrap();
    let yaw = interface.get_f32(tank::GPS_MODULE, 5).unwrap();

    let goal_x = goal.0;
    let goal_y = goal.1;
    let _goal_yaw = goal.2;

    let dx = goal_x - x;
    let dy = goal_y - y;

    let desired_orient = dy.atan2(dx);

    let mut left = 0.0;
    let mut right = 0.0;

    fn angle_diff(a: f32, b: f32) -> f32 {
        let a = a - b;
        (a + std::f32::consts::PI).rem_euclid(std::f32::consts::PI * 2.0) - std::f32::consts::PI
    }
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

    interface.set_f32(tank::BASE_MODULE, 2, left).unwrap();
    interface.set_f32(tank::BASE_MODULE, 3, right).unwrap();
}

pub fn stop(interface: &mut dyn Interface) {
    interface.set_f32(tank::BASE_MODULE, 2, 0.0).unwrap();
    interface.set_f32(tank::BASE_MODULE, 3, 0.0).unwrap();
}
