//! Receiver for radio messages.

/// The minimum selectable channel, integer value.
pub const REG_CHANNEL_MIN: u32 = 0x12;
/// The maximum selectable channel, integer value.
pub const REG_CHANNEL_MAX: u32 = 0x13;

/// The currently selected channel, integer value.
pub const REG_CHANNEL_SELECT: u32 = 0x1000;

/// The payload count. Set to 0 to clear all messages, int value.
pub const REG_PAYLOAD_COUNT: u32 = 0x2000;
/// The payload start offset.
pub const REG_PAYLOAD_START: u32 = 0x2001;
/// The strength offset, float value.
pub const REG_PAYLOAD_OFFSET_STRENGTH: u32 = 0;
/// The data offset, bytes value.
pub const REG_PAYLOAD_OFFSET_DATA: u32 = 1;
/// The stride of payloads.
pub const REG_PAYLOAD_STRIDE: u32 = 1;
