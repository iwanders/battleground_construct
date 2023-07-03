//! Holds module constants specific to the constructor unit.

/// Module identifier for the differential drive module.
pub const MODULE_CONSTRUCTOR_DIFF_DRIVE: u32 = 0x1000;

/// Module identifier for the revolute joint to rotate the turret yaw.
pub const MODULE_CONSTRUCTOR_REVOLUTE_STEER: u32 = 0x1100;

/// Distance in z between the floor and the body center.
pub const CONSTRUCTOR_DIM_FLOOR_TO_BODY_Z: f32 = 0.25;
