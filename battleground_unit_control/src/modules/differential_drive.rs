//! Controls a differential drive base platform.

/// The current left wheel/track velocity, float value.
pub const REG_LEFT_VEL: u32 = 0;
/// The current right wheel/track velocity, float value.
pub const REG_RIGHT_VEL: u32 = 1;

/// The commanded left wheel/track velocity, float value.
pub const REG_LEFT_CMD: u32 = 2;
/// The commanded right wheel/track velocity, float value.
pub const REG_RIGHT_CMD: u32 = 3;
