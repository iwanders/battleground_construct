pub mod registers {
    pub const TRANSMIT_RANGE_MAX: u32 = 0x10;
    pub const TRANSMIT_INTERVAL: u32 = 0x11;

    pub const TRANSMIT_STRENGTH_MAX: u32 = 0x12;

    pub const PAYLOAD_SIZE_LIMIT: u32 = 0x14;
    pub const PAYLOAD_COUNT_LIMIT: u32 = 0x15;

    pub const TRANSMIT_STRENGTH: u32 = 0x1001;
    pub const PAYLOAD_COUNT: u32 = 0x2000;
    pub const PAYLOAD_OFFSET: u32 = 0x2001;
}
