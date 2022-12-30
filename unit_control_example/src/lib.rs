use battleground_unit_control::{log, Interface, RegisterType, UnitControl};
use std::f32::consts::PI;

use battleground_unit_control::modules::cannon::*;
use battleground_unit_control::modules::revolute::*;
use battleground_unit_control::units::tank;

#[derive(Default)]
pub struct SimpleUnitControl {}

impl SimpleUnitControl {
    pub fn new() -> Self {
        SimpleUnitControl {}
    }
}

fn _bar() {
    println!("In bar");
    panic!("buhuuuu");
}

fn _foo() {
    println!("In foo");
    _bar();
}

impl UnitControl for SimpleUnitControl {
    fn update(&mut self, interface: &mut dyn Interface) -> Result<(), Box<dyn std::error::Error>> {
        // test error propagation.
        // panic!();
        // let sparkle_heart = vec![240, 159, 146, 150];
        // let _sparkle_heart = vec![0, 0, 146, 150];
        // let _sparkle_heart = String::from_utf8(_sparkle_heart)?;
        // loop {
        // log::info!("We got called");
        // }
        _foo();

        log::info!("We got called");

        // just try to fire all the time.
        let write_res =
            interface.set_i32(tank::MODULE_TANK_CANNON, REG_CANNON_TRIGGER, true as i32);
        write_res.unwrap();

        let clock = interface.get_f32(tank::MODULE_TANK_CLOCK, 0).unwrap();
        if clock < 0.1 {
            interface
                .set_f32(
                    tank::MODULE_TANK_REVOLUTE_TURRET,
                    REG_REVOLUTE_VELOCITY_CMD,
                    0.3,
                )
                .unwrap();
            interface
                .set_f32(
                    tank::MODULE_TANK_REVOLUTE_BARREL,
                    REG_REVOLUTE_VELOCITY_CMD,
                    -0.1,
                )
                .unwrap();
            return Ok(());
        }

        // println!("Clock: {clock}");
        // base
        // interface.set_f32(0x1000, 2, 1.0).unwrap();
        // interface.set_f32(0x1000, 3, 1.0).unwrap();

        let turret_pos = interface
            .get_f32(tank::MODULE_TANK_REVOLUTE_TURRET, REG_REVOLUTE_POSITION)
            .unwrap();
        // println!("turret_pos: {turret_pos}");
        if (turret_pos > PI && turret_pos < (PI * 2.0 - PI / 8.0))
            || (turret_pos > PI / 8.0 && turret_pos < PI)
        {
            interface
                .set_f32(
                    tank::MODULE_TANK_REVOLUTE_TURRET,
                    REG_REVOLUTE_VELOCITY_CMD,
                    -interface
                        .get_f32(tank::MODULE_TANK_REVOLUTE_TURRET, REG_REVOLUTE_VELOCITY)
                        .unwrap(),
                )
                .unwrap();
        }

        let barrel_pos = interface
            .get_f32(tank::MODULE_TANK_REVOLUTE_BARREL, REG_REVOLUTE_POSITION)
            .unwrap();
        // println!("barrel_pos: {barrel_pos}");
        if barrel_pos < (PI * 2.0 - PI / 8.0) || (barrel_pos < PI / 8.0) {
            interface
                .set_f32(
                    tank::MODULE_TANK_REVOLUTE_BARREL,
                    REG_REVOLUTE_VELOCITY_CMD,
                    -interface
                        .get_f32(tank::MODULE_TANK_REVOLUTE_BARREL, REG_REVOLUTE_VELOCITY)
                        .unwrap(),
                )
                .unwrap();
        }

        if false {
            for m_index in interface.modules().unwrap() {
                log::info!(
                    "update, module name: {}",
                    interface.module_name(m_index).unwrap()
                );
                log::info!("obtaining registers list for {m_index}.");
                let v = interface.registers(m_index);
                log::info!("{:?}", v);
                for r_index in v.unwrap() {
                    log::info!("  {}", interface.register_name(m_index, r_index).unwrap());
                    let register_type = interface.register_type(m_index, r_index).unwrap();
                    log::info!("    {:?}", register_type);
                    match register_type {
                        RegisterType::I32 => {
                            let v = interface.get_i32(m_index, r_index).unwrap();
                            log::info!("   -> {v}");
                            interface.set_i32(m_index, r_index, v + 1).unwrap();
                            let v = interface.get_i32(m_index, r_index).unwrap();
                            log::info!("   -> {v}");
                        }
                        RegisterType::F32 => {
                            let v = interface.get_f32(m_index, r_index).unwrap();
                            log::info!("   -> {v}");
                            interface.set_f32(m_index, r_index, v + 1.5).unwrap();
                            let v = interface.get_f32(m_index, r_index).unwrap();
                            log::info!("   -> {v}");
                            // let v = interface.get_f32(m_index, r_index).unwrap();
                            // log::info!("   -> {v}");
                        }
                        RegisterType::Bytes => {
                            let len = interface.get_bytes_len(m_index, r_index).unwrap();
                            log::info!("Bytes len: {len:?}");
                            let v = [0, 1, 2, 3u8];
                            interface.set_bytes(m_index, r_index, &v).unwrap();
                            let len = interface.get_bytes_len(m_index, r_index).unwrap();
                            log::info!("Bytes len: {len:?}");
                            let mut read_v = [0, 0, 0, 3u8];
                            interface.get_bytes(m_index, r_index, &mut read_v).unwrap();
                            log::info!("read_v len: {read_v:?}");

                            // let v = interface.get_f32(m_index, r_index).unwrap();
                            // log::info!("   -> {v}");
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

#[no_mangle]
#[cfg(target_arch = "wasm32")]
pub fn create_unit_control() -> Box<dyn UnitControl> {
    // Box::new(battleground_construct::control::radar_draw::RadarDrawControl{})
    // Box::new(battleground_construct::control::tank_swivel_shoot::TankSwivelShoot {})
    Box::new(SimpleUnitControl::new())
}
