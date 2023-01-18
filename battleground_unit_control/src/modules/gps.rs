//! The global position of the unit.
//!
//! This gps is usually attached to the center of the units' body.

/// Register holding the x coordinate in meters, float value.
pub const REG_GPS_X: u32 = 0;
/// Register holding the y coordinate in meters, float value.
pub const REG_GPS_Y: u32 = 1;
/// Register holding the z coordinate in meters, float value.
pub const REG_GPS_Z: u32 = 2;
/// Register holding the roll, float value, radians.
pub const REG_GPS_ROLL: u32 = 3;
/// Register holding the pitch, float value, radians.
pub const REG_GPS_PITCH: u32 = 4;
/// Register holding the yaw, float value, radians.
pub const REG_GPS_YAW: u32 = 5;
