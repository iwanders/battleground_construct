//! Fire control of gun batteries (like the artillery)
//!
//! Gun batteries have multiple guns, between firing each gun there is an inter gun delay.
//! Each individual gun has its own reload duration, there may be an additional gun battery reload
//! duration when all guns have been emptied.
//! Individual guns have their own pose, last reload time and ready state. Guns fire in order.

/// Set the gun battery to be firing. This does not disable when a shot is fired, but keeps firing.
pub const REG_GUN_BATTERY_FIRING: u32 = 0;

/// Denotes if the gun battery has been triggered.
pub const REG_GUN_BATTERY_IS_TRIGGERED: u32 = 1;

/// Denotes if the gun battery is ready to fire, boolean value.
pub const REG_GUN_BATTERY_READY: u32 = 2;

/// Provides the reload time per gun in seconds.
pub const REG_GUN_BATTERY_GUN_RELOAD: u32 = 3;

/// Provides the duration in between individual gun firings.
pub const REG_GUN_BATTERY_INTER_GUN_DURATION: u32 = 4;

/// Provides the reload time for the entire battery (after it has been emptied) in seconds.
pub const REG_GUN_BATTERY_RELOAD: u32 = 5;

/// The index of the gun that will fire next.
pub const REG_GUN_BATTERY_FIRE_INDEX: u32 = 6;

/// The number of guns in this battery.
pub const REG_GUN_BATTERY_COUNT: u32 = 0x1000;
/// The start of the gun list.
pub const REG_GUN_BATTERY_START: u32 = 0x1001;

/// The offset of the gun's x value, float value.
pub const REG_GUN_BATTERY_OFFSET_X: u32 = 0;
/// The offset of the gun's y value, float value.
pub const REG_GUN_BATTERY_OFFSET_Y: u32 = 1;
/// The offset of the gun's z value, float value.
pub const REG_GUN_BATTERY_OFFSET_Z: u32 = 2;

/// The offset of the roll value, float value, radians.
pub const REG_GUN_BATTERY_OFFSET_ROLL: u32 = 3;
/// The offset of the pitch value, float value, radians.
pub const REG_GUN_BATTERY_OFFSET_PITCH: u32 = 4;
/// The offset of the yaw value, float value, radians.
pub const REG_GUN_BATTERY_OFFSET_YAW: u32 = 5;

/// The offset of the last fire time value, float value, seconds.
pub const REG_GUN_BATTERY_OFFSET_LAST_FIRE_TIME: u32 = 6;
/// Whether this gun is currently ready for fire, integer (bool) value.
pub const REG_GUN_BATTERY_OFFSET_READY: u32 = 7;

/// The stride of each reflection record.
pub const REG_GUN_BATTERY_STRIDE: u32 = 8;
