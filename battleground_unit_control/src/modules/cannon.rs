//! Fire control of simple cannons.

/// Trigger the cannon to fire, is_triggered will go true, and the cannon will fire as soon as ready
/// then the trigger becomes inactive until activated again.
pub const REG_CANNON_TRIGGER: u32 = 0;

/// Denotes if the cannon has been triggered.
pub const REG_CANNON_IS_TRIGGERED: u32 = 1;

/// Denotes if the cannon is ready to fire, boolean value.
pub const REG_CANNON_READY: u32 = 2;

/// Provides the reload time in seconds, float value.
pub const REG_CANNON_RELOAD_TIME: u32 = 3;
