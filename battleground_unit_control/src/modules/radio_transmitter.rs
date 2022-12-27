//! Transmitter for radio messages.

//!
//! A radio sends messages at least [`REG_TRANSMIT_INTERVAL`] apart.

/// The maximum transmit range for the radio, float value.
pub const REG_TRANSMIT_RANGE_MAX: u32 = 0x10;
/// The transmission interval for the radio, float value.
pub const REG_TRANSMIT_INTERVAL: u32 = 0x11;

/// The minimum selectable channel, integer value.
pub const REG_CHANNEL_MIN: u32 = 0x12;
/// The maximum selectable channel, integer value.
pub const REG_CHANNEL_MAX: u32 = 0x13;

/// The maximum payload size, integer value.
pub const REG_PAYLOAD_SIZE_LIMIT: u32 = 0x14;
/// The maximum number of pending transmissions, integer value.
pub const REG_PAYLOAD_COUNT_LIMIT: u32 = 0x15;

/// The currently selected channel, integer value.
pub const REG_CHANNEL_SELECT: u32 = 0x1000;

/// The current number of pending transmissions, integer value.
pub const REG_PAYLOAD_COUNT: u32 = 0x2000;
/// The start of pending transmissions, byte values.
pub const REG_PAYLOAD_START: u32 = 0x2001;
