pub mod registers {

    pub const RANGE_MAX: u32 = 0x10;
    pub const DETECTION_ANGLE_YAW: u32 = 0x11;
    pub const DETECTION_ANGLE_PITCH: u32 = 0x12;

    pub const REFLECTION_COUNT: u32 = 0x1000;
    pub const REFLECTION_START: u32 = 0x1001;
    pub const REFLECTION_OFFSET_YAW: u32 = 0;
    pub const REFLECTION_OFFSET_PITCH: u32 = 1;
    pub const REFLECTION_OFFSET_DISTANCE: u32 = 2;
    pub const REFLECTION_OFFSET_STRENGTH: u32 = 3;
    pub const REFLECTION_STRIDE: u32 = 4;
}
