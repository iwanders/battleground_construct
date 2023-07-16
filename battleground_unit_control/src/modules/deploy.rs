//! Controls the deployment process.
//!

/// When the deploy state matches the desired state.
pub const REG_DEPLOY_FINISHED: u32 = 0;

/// Register to hold the desired deploy state.
pub const REG_DEPLOY_DESIRED_STATE: u32 = 1;

/// Register to hold the current deployment state.
pub const REG_DEPLOY_STATE: u32 = 1;

/// Value used to express normal state.
pub const DEPLOY_STATE_NORMAL: i32 = 0;
/// Value used to express deployed state.
pub const DEPLOY_STATE_DEPLOYED: i32 = 1;
/// Value used to express transition in progress state.
pub const DEPLOY_STATE_IN_TRANSITION: i32 = 2;
