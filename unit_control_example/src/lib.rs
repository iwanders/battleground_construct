// We import the logging module for printing, println! won't work inside a wasm plugin.
use battleground_unit_control::log;
// We import the Interface trait and UnitControl trait, we need to implement UnitControl and will
// control our unit through the Interface.
use battleground_unit_control::{Interface, UnitControl};

// Modules hold their register index constants, they always contain the module name so they
// can be imported without collisions.
use battleground_unit_control::modules::{
    clock::*, differential_drive::*, draw::*, gps::*, revolute::*, unit::*,
};
// Module constants live in common and their respective units.
use battleground_unit_control::units::{common, tank, UnitType};

/// Our example controller!
pub struct UnitControlExample {
    last_print: f32,
}
impl Default for UnitControlExample {
    fn default() -> Self {
        Self {
            last_print: -10000.0,
        }
    }
}

impl UnitControl for UnitControlExample {
    /// This function gets called periodically to control our unit.
    fn update(&mut self, interface: &mut dyn Interface) -> Result<(), Box<dyn std::error::Error>> {
        // This gets the current time.
        let t = interface.get_f32(common::MODULE_CLOCK, REG_CLOCK_ELAPSED)?;

        // This can be used to retrieve the unit type.
        let unit_type = interface.get_i32(common::MODULE_UNIT, REG_UNIT_UNIT_TYPE)?;
        let unit_type: UnitType = (unit_type as u32).try_into()?;

        // Every 15 seconds, dump all registers and what unit we are.
        if (self.last_print + 15.0) < t {
            self.last_print = t;
            log::info!("I'm a {unit_type:?}");
            dump_registers(interface)?;
        }

        // If we are a tank, lets rotate in place to show how to command the tracks.
        if unit_type == UnitType::Tank {
            interface.set_f32(tank::MODULE_TANK_DIFF_DRIVE, REG_DIFF_DRIVE_LEFT_CMD, 0.1)?;
            interface.set_f32(tank::MODULE_TANK_DIFF_DRIVE, REG_DIFF_DRIVE_RIGHT_CMD, -0.1)?;
        }

        // Show how to draw things, that's super useful when debugging.
        draw_stuff(interface)?;

        Ok(())
    }
}

fn dump_registers(interface: &dyn Interface) -> Result<(), Box<dyn std::error::Error>> {
    use battleground_unit_control::RegisterType;
    for module_index in interface.modules()? {
        log::info!("module_name: {}", interface.module_name(module_index)?);
        let registers = interface.registers(module_index)?;
        for register_index in registers {
            let name = interface.register_name(module_index, register_index)?;
            let register_type = interface.register_type(module_index, register_index)?;
            match register_type {
                RegisterType::I32 => {
                    let v = interface.get_i32(module_index, register_index)?;
                    log::info!("    {module_index:0>8x}:{register_index:0>8x} (i32) {name}: {v}");
                }
                RegisterType::F32 => {
                    let v = interface.get_f32(module_index, register_index)?;
                    log::info!("    {module_index:0>8x}:{register_index:0>8x} (f32) {name}: {v}");
                }
                RegisterType::Bytes => {
                    let len = interface.get_bytes_len(module_index, register_index)?;
                    let mut read_v = vec![0; len];
                    interface.get_bytes(module_index, register_index, &mut read_v)?;
                    log::info!("    {module_index:0>8x}:{register_index:0>8x} (bytes {len}) {name}: {read_v:?}");
                }
            }
        }
    }
    Ok(())
}

fn draw_stuff(interface: &mut dyn Interface) -> Result<(), Box<dyn std::error::Error>> {
    use std::f32::consts::PI;

    // get the location of our unit.
    let x = interface.get_f32(common::MODULE_GPS, REG_GPS_X)?;
    let y = interface.get_f32(common::MODULE_GPS, REG_GPS_Y)?;
    let z = interface.get_f32(common::MODULE_GPS, REG_GPS_Z)?;
    let yaw = interface.get_f32(common::MODULE_GPS, REG_GPS_YAW)?;

    // Vector for the lines we're going to draw.
    let mut lines = vec![];

    const GREEN: [u8; 4] = [0, 255, 0, 255];
    const BLUE: [u8; 4] = [0, 0, 255, 255];
    const RED: [u8; 4] = [255, 0, 0, 255];
    const TRANSPARENT_MAGENTA: [u8; 4] = [255, 0, 255, 64];

    // Red line from the origin to the vehicle.
    lines.push(LineSegment {
        p0: [0.0, 0.0, z],
        p1: [x, y, z],
        width: 0.05,
        color: RED,
    });

    // Blue line pointing out of the front of the vehicle.
    lines.push(LineSegment {
        p0: [x, y, z],
        p1: [x + yaw.cos() * 5.0, y + yaw.sin() * 5.0, z],
        width: 0.05,
        color: BLUE,
    });

    // Green line for yaw + radar (this does not account for turret, so it is wrong!)
    // Also assume it is a tank, unwrapping to 0.0 if it isn't.
    let radar_pos = interface
        .get_f32(tank::MODULE_TANK_REVOLUTE_RADAR, REG_REVOLUTE_POSITION)
        .unwrap_or(0.0);
    lines.push(LineSegment {
        p0: [x, y, z + 0.5],
        p1: [
            x + (yaw + radar_pos).cos() * 5.0,
            y + (yaw + radar_pos).sin() * 5.0,
            z + 0.5,
        ],
        width: 0.05,
        color: GREEN,
    });

    // Finally, draw a circle around our tank, just because we can.
    for i in 1..20 {
        let now = ((i as f32) / 19.0) * 2.0 * PI;
        let prev = (((i - 1) as f32) / 19.0) * 2.0 * PI;
        let r = 3.5;
        lines.push(LineSegment {
            p0: [x + now.cos() * r, y + now.sin() * r, z + 0.5],
            p1: [x + prev.cos() * r, y + prev.sin() * r, z + 0.5],
            width: 0.05,
            color: TRANSPARENT_MAGENTA,
        });
    }

    // Now we just need to convert the drawing structs to bytes and set the draw register.
    let mut draw_instructions: Vec<u8> = vec![];
    for l in lines {
        draw_instructions.extend(l.into_le_bytes());
    }
    interface.set_bytes(
        common::MODULE_DRAW,
        battleground_unit_control::modules::draw::REG_DRAW_LINES,
        &draw_instructions,
    )?;

    Ok(())
}

#[no_mangle]
#[cfg(target_arch = "wasm32")]
pub fn create_unit_control() -> Box<dyn UnitControl> {
    Box::new(UnitControlExample::default())
}
