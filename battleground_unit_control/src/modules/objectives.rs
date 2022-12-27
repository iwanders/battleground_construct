pub mod registers {
    pub const CAPTURE_POINT_COUNT: u32 = 0x1000;

    pub const CAPTURE_POINT_START: u32 = 0x1001;
    pub const CAPTURE_POINT_OFFSET_X: u32 = 0;
    pub const CAPTURE_POINT_OFFSET_Y: u32 = 1;
    pub const CAPTURE_POINT_OFFSET_OWNER: u32 = 2;
    pub const CAPTURE_POINT_OFFSET_RADIUS: u32 = 3;
    pub const CAPTURE_POINT_STRIDE: u32 = 4;

    pub const CAPTURE_POINT_UNOWNED: i32 = -1;
}
