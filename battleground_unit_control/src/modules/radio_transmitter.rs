//! Transmitter for radio messages.
//!
//! A radio sends messages at least [`REG_RADIO_TX_INTERVAL`] apart. The radio sends messages on the
//! selected channel only. The messages have a size limit and the maximum number of messages stored
//! is also limited.
//! The first message in the list is the first one to be sent, they are sent in order. You can
//! change or rewrite the message list as you see fit.

/// The maximum transmit range for the radio, float value.
pub const REG_RADIO_TX_RANGE_MAX: u32 = 0x10;
/// The transmission interval for the radio, float value.
pub const REG_RADIO_TX_INTERVAL: u32 = 0x11;

/// The minimum selectable channel, integer value.
pub const REG_RADIO_TX_CHANNEL_MIN: u32 = 0x12;
/// The maximum selectable channel, integer value.
pub const REG_RADIO_TX_CHANNEL_MAX: u32 = 0x13;

/// The maximum payload size, integer value, messages are shortened to this length.
pub const REG_RADIO_TX_MSG_SIZE_LIMIT: u32 = 0x14;
/// The maximum number of pending transmissions, integer value.
pub const REG_RADIO_TX_MSG_COUNT_LIMIT: u32 = 0x15;

/// The currently selected channel, integer value.
pub const REG_RADIO_TX_CHANNEL_SELECT: u32 = 0x1000;

/// The current number of pending transmissions, integer value.
pub const REG_RADIO_TX_MSG_COUNT: u32 = 0x2000;
/// The start of pending transmissions, byte values.
pub const REG_RADIO_TX_MSG_START: u32 = 0x2001;
