//! Controller information.
//!
//! Provides information about the controller that is used for this unit.

/// Returns the duration between controller updates, float value in seconds.
pub const REG_CONTROLLER_UPDATE_INTERVAL: u32 = 0;

/// Returns whether cpu fuel is enabled.
/// Register is only available when ran inside a wasm controller.
pub const REG_CONTROLLER_WASM_CPU_FUEL_ENABLED: u32 = 0x1000;
/// Returns the amount of cpu fuel remaining for this invocation of the controller.
/// Register is only available when ran inside a wasm controller.
pub const REG_CONTROLLER_WASM_CPU_FUEL_LEFT: u32 = 0x1001;
