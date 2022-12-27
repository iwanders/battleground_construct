//! Information about radar reflections.

//!
//! The radar returns reflections in the radar frame, detection is limited by the detection yaw
//! and detection pitch, both in the radar's frame.

/// The maximum detection range of the radar, float value.
pub const REG_RADAR_RANGE_MAX: u32 = 0x10;

/// The maximum detection yaw of the radar, float value, radians.
pub const REG_RADAR_DETECTION_ANGLE_YAW: u32 = 0x11;
/// The maximum detection pitch of the radar, float value, radians.
pub const REG_RADAR_DETECTION_ANGLE_PITCH: u32 = 0x12;

/// The number of reflection records.
pub const REG_RADAR_REFLECTION_COUNT: u32 = 0x1000;
/// The start of the reflection list.
pub const REG_RADAR_REFLECTION_START: u32 = 0x1001;

/// The offset of the yaw value, float value, radians.
pub const REG_RADAR_REFLECTION_OFFSET_YAW: u32 = 0;
/// The offset of the pitch value, float value, radians.
pub const REG_RADAR_REFLECTION_OFFSET_PITCH: u32 = 1;
/// The offset of the distance value, float value.
pub const REG_RADAR_REFLECTION_OFFSET_DISTANCE: u32 = 2;
/// The offset of the strength value, float value.
pub const REG_RADAR_REFLECTION_OFFSET_STRENGTH: u32 = 3;
/// The stride of each reflection record.
pub const REG_RADAR_REFLECTION_STRIDE: u32 = 4;
