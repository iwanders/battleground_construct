//! Revolute joint module

//!
//! Revolute joints rotate about any child component about their specified axis of rotation.
//! Only the rotation velocity can be controlled.

/// The current rotation of the revolute joint, float value, radians.
pub const REG_POSITION: u32 = 0;
/// The current rotational velocity of the revolute joint, float value, radians per time.
pub const REG_VELOCITY: u32 = 1;
/// The minimum rotational velocity of the revolute joint, float value.
pub const REG_VELOCITY_MIN: u32 = 2;
/// The maximum rotational velocity of the revolute joint, float value.
pub const REG_VELOCITY_MAX: u32 = 3;
/// The commanded rotational velocity of the revolute joint, float value, radians per time.
pub const REG_VELOCITY_CMD: u32 = 4;
