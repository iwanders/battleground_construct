pub const MODULE_ARTILLERY_DIFF_DRIVE: u32 = 0x1000;
pub const MODULE_ARTILLERY_REVOLUTE_TURRET: u32 = 0x1100;
pub const MODULE_ARTILLERY_REVOLUTE_BARREL: u32 = 0x1200;
pub const MODULE_ARTILLERY_GUN_BATTERY: u32 = 0x1300;
pub const MODULE_ARTILLERY_REVOLUTE_RADAR: u32 = 0x1500;
pub const MODULE_ARTILLERY_RADAR: u32 = 0x1600;

/// Distance in z between the floor and the body center.
pub const ARTILLERY_DIM_FLOOR_TO_BODY_Z: f32 = 0.25;
/// Distance in z between the floor and the turret center (and center of rotation).
pub const ARTILLERY_DIM_FLOOR_TO_TURRET_Z: f32 = 0.375;
/// Distance between the turret and barrel joint, in local frame.
pub const ARTILLERY_DIM_TURRET_TO_BARREL_Z: f32 = 0.6;
/// Distance between the turret and the radar joint.
pub const ARTILLERY_DIM_TURRET_TO_RADAR_Z: f32 = 0.05;
/// Distance between the radar rotation and the actual radar
pub const ARTILLERY_DIM_RADAR_JOINT_TO_RADAR_X: f32 = 0.4;
/// Distance between the barrel joint and the muzzle (barrel length).
pub const ARTILLERY_DIM_BARREL_TO_MUZZLE_X: f32 = 0.4;

/// Velocity at which cannon bullets exit the ARTILLERY's muzzle.
pub const ARTILLERY_PARAM_MUZZLE_VELOCITY: f32 = 10.0;
