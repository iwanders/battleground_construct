pub mod registers {
    pub const CHANNEL_MIN: u32 = 0x12;
    pub const CHANNEL_MAX: u32 = 0x13;

    pub const CHANNEL_SELECT: u32 = 0x1000;

    pub const PAYLOAD_COUNT: u32 = 0x2000;
    pub const PAYLOAD_START: u32 = 0x2001;
    pub const PAYLOAD_OFFSET_STRENGTH: u32 = 0;
    pub const PAYLOAD_OFFSET_DATA: u32 = 1;
    pub const PAYLOAD_STRIDE: u32 = 1;
}
