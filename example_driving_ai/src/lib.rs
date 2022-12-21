use battleground_vehicle_control::{log, Interface, RegisterType, VehicleControl};

#[derive(Default)]
pub struct SimpleVehicleControl {}

impl SimpleVehicleControl {
    pub fn new() -> Self {
        SimpleVehicleControl {}
    }
}

impl VehicleControl for SimpleVehicleControl {
    fn update(&mut self, interface: &mut dyn Interface) {
        log::info!("We got called");

        if true {
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
    }
}

#[no_mangle]
#[cfg(target_arch = "wasm32")]
pub fn create_vehicle_control() -> Box<dyn VehicleControl> {
    // Box::new(battleground_construct::control::radar_draw::RadarDrawControl{})
    // Box::new(battleground_construct::control::tank_swivel_shoot::TankSwivelShoot {})
    Box::new(SimpleVehicleControl::new())
}
