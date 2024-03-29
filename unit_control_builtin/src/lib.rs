pub mod diff_drive_capturable;
pub mod diff_drive_forwards_backwards;
mod diff_drive_util;

#[cfg(not(target_arch = "wasm32"))]
pub mod dynamic_load_control;

pub mod idle;
pub mod interface_printer;
pub mod naive_shoot;
pub mod tank_swivel_shoot;

pub mod radio_position;
pub mod sequence_control;

pub type UnitControlResult = Result<(), Box<dyn std::error::Error>>;
