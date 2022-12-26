pub mod diff_drive_forwards_backwards;

#[cfg(not(target_arch = "wasm32"))]
pub mod dynamic_load_control;

pub mod idle;
pub mod tank_swivel_shoot;
