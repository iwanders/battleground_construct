use battleground_unit_control::{ControlError, Interface, RegisterType, UnitControl};

pub struct InterfacePrinter {}
impl UnitControl for InterfacePrinter {
    fn update(&mut self, interface: &mut dyn Interface) -> Result<(), Box<ControlError>> {
        for m_index in interface.modules().unwrap() {
            println!("- {}", interface.module_name(m_index).unwrap());
            let mut v = interface.registers(m_index).unwrap();
            v.sort_by(|a, b| a.cmp(b)); // sort for display.
            println!(" Registers: {:x?}:", v);
            for r_index in v {
                print!("  ({})", interface.register_name(m_index, r_index).unwrap());
                let register_type = interface.register_type(m_index, r_index).unwrap();
                print!("[{:?}]", register_type);
                match register_type {
                    RegisterType::I32 => {
                        let v = interface.get_i32(m_index, r_index).unwrap();
                        print!(" {v}");
                    }
                    RegisterType::F32 => {
                        let v = interface.get_f32(m_index, r_index).unwrap();
                        print!(" {v}");
                    }
                    RegisterType::Bytes => {
                        let len = interface.get_bytes_len(m_index, r_index).unwrap();
                        print!("#{len:?}");
                        let mut read_v = vec![0; len];
                        interface.get_bytes(m_index, r_index, &mut read_v).unwrap();
                        print!(" {read_v:?}");
                    }
                }
                println!();
            }
            println!();
        }
        Ok(())
    }
}
