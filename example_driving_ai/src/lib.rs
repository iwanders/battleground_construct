use battleground_vehicle_control::{log, Interface, VehicleControl};

pub struct SimpleAi {}

impl SimpleAi {
    pub fn new() -> Self {
        SimpleAi {}
    }
}

impl VehicleControl for SimpleAi {
    fn update(&mut self, interface: &mut dyn Interface) {
        log::info!("We got called");

        if true {
            for m_index in interface.modules().unwrap() {
                log::info!(
                    "update, module name: {}",
                    interface.module_name(m_index).unwrap()
                );
                for r_index in interface.registers(m_index).unwrap() {
                    log::info!("  {}", interface.register_name(m_index, r_index).unwrap());
                }
            }
        }
    }
}

#[no_mangle]
#[cfg(target_arch = "wasm32")]
pub fn create_vehicle_control() -> Box<dyn VehicleControl> {
    // Box::new(battleground_construct::control::radar_draw::RadarDrawControl{})
    Box::new(SimpleAi::new())
}
