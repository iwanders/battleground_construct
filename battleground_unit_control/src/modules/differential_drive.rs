//! Controls a differential drive base platform.
//!
//! Differential drives have two parallel wheels with a track_width in between. The wheels respect
//! the acceleration limits.

/// The current left wheel/track velocity, float value.
pub const REG_DIFF_DRIVE_LEFT_VEL: u32 = 0;
/// The current right wheel/track velocity, float value.
pub const REG_DIFF_DRIVE_RIGHT_VEL: u32 = 1;

/// The commanded left wheel/track velocity, float value.
pub const REG_DIFF_DRIVE_LEFT_CMD: u32 = 2;
/// The commanded right wheel/track velocity, float value.
pub const REG_DIFF_DRIVE_RIGHT_CMD: u32 = 3;

/// The distance between the left and right wheel/track, float value.
pub const REG_DIFF_DRIVE_TRACK_WIDTH: u32 = 4;

/// The limit on the acceleration lower bound, float value.
pub const REG_DIFF_DRIVE_ACCELERATION_LOWER: u32 = 5;
/// The limit on the acceleration upper bound, float value.
pub const REG_DIFF_DRIVE_ACCELERATION_UPPER: u32 = 6;
