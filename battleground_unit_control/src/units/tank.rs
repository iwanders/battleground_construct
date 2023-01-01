pub const MODULE_TANK_DIFF_DRIVE: u32 = 0x1000;
pub const MODULE_TANK_REVOLUTE_TURRET: u32 = 0x1100;
pub const MODULE_TANK_REVOLUTE_BARREL: u32 = 0x1200;
pub const MODULE_TANK_CANNON: u32 = 0x1300;
pub const MODULE_TANK_REVOLUTE_RADAR: u32 = 0x1500;
pub const MODULE_TANK_RADAR: u32 = 0x1600;

/// Distance in z between the floor and the body center.
pub const TANK_DIM_FLOOR_TO_BODY_Z: f32 = 0.25;
/// Distance in z between the floor and the turret center (and center of rotation).
pub const TANK_DIM_FLOOR_TO_TURRET_Z: f32 = 0.375 + 0.1 / 2.0;
/// Distance between the turret and barrel joint, in local frame.
pub const TANK_DIM_TURRET_TO_BARREL_X: f32 = 0.25;
/// Distance between the turret and the radar joint.
pub const TANK_DIM_TURRET_TO_RADAR_Z: f32 = 0.07;
/// Distance between the barrel joint and the muzzle (barrel length).
pub const TANK_DIM_BARREL_TO_MUZZLE_X: f32 = 1.0;

/// Velocity at which cannon bullets exit the tank's muzzle.
pub const TANK_PARAM_MUZZLE_VELOCITY: f32 = 10.0;
