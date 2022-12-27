//! Receiver for radio messages.

/// The minimum selectable channel, integer value.
pub const REG_RADIO_RX_CHANNEL_MIN: u32 = 0x12;
/// The maximum selectable channel, integer value.
pub const REG_RADIO_RX_CHANNEL_MAX: u32 = 0x13;

/// The currently selected channel, integer value.
pub const REG_RADIO_RX_CHANNEL_SELECT: u32 = 0x1000;

/// The MSG count. Set to 0 to clear all messages, int value.
pub const REG_RADIO_RX_MSG_COUNT: u32 = 0x2000;
/// The MSG start offset.
pub const REG_RADIO_RX_MSG_START: u32 = 0x2001;
/// The strength offset, float value.
pub const REG_RADIO_RX_MSG_OFFSET_STRENGTH: u32 = 0;
/// The data offset, bytes value.
pub const REG_RADIO_RX_MSG_OFFSET_DATA: u32 = 1;
/// The stride of MSGs.
pub const REG_RADIO_RX_MSG_STRIDE: u32 = 1;
