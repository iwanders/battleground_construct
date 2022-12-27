//! Objective information.

/// Register index to obtain the capture point count.
pub const REG_CAPTURE_POINT_COUNT: u32 = 0x1000;

/// Register start index for the capture point list.
pub const REG_CAPTURE_POINT_START: u32 = 0x1001;

/// Register offset for the x position of a capture point, float value.
pub const REG_CAPTURE_POINT_OFFSET_X: u32 = 0;
/// Register offset for the y position of a capture point, float value.
pub const REG_CAPTURE_POINT_OFFSET_Y: u32 = 1;
/// Register offset for the owner of a capture point, integer value.
pub const REG_CAPTURE_POINT_OFFSET_OWNER: u32 = 2;
/// Register offset for the radius of a capture point, float value.
pub const REG_CAPTURE_POINT_OFFSET_RADIUS: u32 = 3;
/// Register stride for each capture point.
pub const REG_CAPTURE_POINT_STRIDE: u32 = 4;

/// Sentinel value used for the owner if a capture point is unused.
pub const CAPTURE_POINT_UNOWNED: i32 = -1;
