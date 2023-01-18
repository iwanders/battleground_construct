//! Receiver for radio messages.
//!
//! This module receives radio messages, it only receives messages on the channel it has selected.
//! It is limited in the number of messages that it can hold, if the buffer is full, new incoming
//! messages get dropped.
//! You can set the message count to zero to clear all messages (to mark them as received)

/// The minimum selectable channel, integer value.
pub const REG_RADIO_RX_CHANNEL_MIN: u32 = 0x12;
/// The maximum selectable channel, integer value.
pub const REG_RADIO_RX_CHANNEL_MAX: u32 = 0x13;

/// The maximum number of incoming messages in the buffer. If messages come in while the buffer is
/// full, the [`REG_RADIO_RX_MSG_OVERFLOW`] register is incremented by one.
pub const REG_RADIO_RX_MSG_COUNT_LIMIT: u32 = 0x15;

/// The currently selected channel, integer value.
pub const REG_RADIO_RX_CHANNEL_SELECT: u32 = 0x1000;

/// Counter that keeps track of how many messages were lost because the buffer was full while a new
/// messsage was being received, integer value, may be cleared.
pub const REG_RADIO_RX_MSG_OVERFLOW: u32 = 0x1001;

/// The MSG count. Set to 0 to clear all messages, int value.
pub const REG_RADIO_RX_MSG_COUNT: u32 = 0x2000;
/// The MSG start offset.
pub const REG_RADIO_RX_MSG_START: u32 = 0x2001;
/// The strength offset, float value.
pub const REG_RADIO_RX_MSG_OFFSET_STRENGTH: u32 = 0;
/// The data offset, bytes value.
pub const REG_RADIO_RX_MSG_OFFSET_DATA: u32 = 1;
/// The stride of MSGs.
pub const REG_RADIO_RX_MSG_STRIDE: u32 = 2;
