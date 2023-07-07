//! Holds module constants specific to the base tricycle platform.

/// Module identifier for the differential drive module.
pub const MODULE_BASE_TRICYCLE_DRIVE: u32 = 0x1000;

/// Module identifier for the revolute joint to rotate the turret yaw.
pub const MODULE_BASE_TRICYCLE_REVOLUTE_STEER: u32 = 0x1100;

/// Distance in z between the floor and the body center.
pub const BASE_TRICYCLE_DIM_FLOOR_TO_BODY_Z: f32 = 0.25;

/// The wheel base of the constructor vehicle.
pub const BASE_TRICYCLE_WHEEL_BASE: f32 = 1.5;
