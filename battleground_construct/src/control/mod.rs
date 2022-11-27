

pub struct DummyVehicleControl {}
impl battleground_vehicle_control::VehicleControl for DummyVehicleControl {
    fn update(&mut self, interface: &mut dyn battleground_vehicle_control::Interface) {
        for m_index in interface.modules().unwrap() {
            println!(
                "update, module name: {}",
                interface.module_name(m_index).unwrap()
            );
            for r_index in interface.registers(m_index).unwrap() {
                println!("  {}", interface.register_name(m_index, r_index).unwrap());
            }
        }
        interface.set_f32(0x0100, 2, 1.0).unwrap();
        interface.set_f32(0x0100, 3, 1.0).unwrap();

        interface.set_f32(0x0200, 4, 1.0).unwrap();
        interface.set_f32(0x0300, 4, -1.0).unwrap();
    }
}

