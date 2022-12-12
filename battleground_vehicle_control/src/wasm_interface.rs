use log;

// https://stackoverflow.com/a/40234666
macro_rules! function {
    () => {{
        fn f() {}
        fn type_name_of<T>(_: T) -> &'static str {
            std::any::type_name::<T>()
        }
        let name = type_name_of(f);
        &name[..name.len() - 3]
    }};
}

#[no_mangle]
pub extern "C" fn wasm_setup() {
    logging::setup();
    controller::setup();
}

#[no_mangle]
pub extern "C" fn wasm_controller_update() {
    controller::update();
}

pub mod controller {
    extern "Rust" {
        fn create_vehicle_control() -> Box<dyn VehicleControl>;
    }
    use crate::*;

    // Not sure if this is 100% properly safe... but we're in wasm land where everything is single
    // threaded.
    use std::ops::DerefMut;
    use std::sync::Mutex;
    struct ControllerWrapper(Box<dyn VehicleControl>);
    unsafe impl std::marker::Send for ControllerWrapper {}

    static CONTROLLER: Mutex<Option<ControllerWrapper>> = Mutex::new(None);

    pub fn setup() {
        log::info!("Allocating control");
        *((CONTROLLER.lock().expect("cant be poisoned")).deref_mut()) =
            unsafe { Some(ControllerWrapper(create_vehicle_control())) };
    }

    pub fn update() {
        let mut static_interface = super::interface::StaticInterface;
        (CONTROLLER.lock().expect("cant be poisoned"))
            .deref_mut()
            .as_mut()
            .unwrap()
            .0
            .update(&mut static_interface);
    }
}

mod interface {
    use crate::*;
    use std::sync::Mutex;
    extern "C" {
        fn wasm_interface_modules() -> u32;
        fn wasm_interface_registers(module: u32) -> u32;
        fn wasm_interface_module_name(module: u32) -> u32;
        fn wasm_interface_register_name(module: u32, register: u32) -> u32;
        fn wasm_interface_register_type(module: u32, register: u32) -> u32;

        fn wasm_interface_get_i32(module: u32, register: u32) -> i32;
        fn wasm_interface_get_f32(module: u32, register: u32) -> f32;
        fn wasm_interface_set_i32(module: u32, register: u32, value: i32) -> i32;
        fn wasm_interface_set_f32(module: u32, register: u32, value: f32) -> f32;

        fn wasm_interface_get_bytes_len(module: u32, register: u32) -> u32;
        fn wasm_interface_get_bytes(module: u32, register: u32, dest: u32, len: u32) -> u32;
        // return type of set bytes is just for uniform handling on the wasm wrapper side.
        fn wasm_interface_set_bytes(module: u32, register: u32, src: u32, len: u32) -> u32;
    }

    static BUFFER: Mutex<Vec<u8>> = Mutex::new(Vec::new());

    #[no_mangle]
    pub extern "C" fn wasm_transmission_buffer(len: u32) -> *mut u8 {
        let mut buffer = BUFFER.lock().expect("cannot be poisoned");
        buffer.clear();
        buffer.resize(len as usize, 0);
        buffer.as_mut_ptr()
    }

    const NO_ERROR: u32 = 0xFFFF;
    static ERROR: Mutex<u32> = Mutex::new(NO_ERROR);
    #[no_mangle]
    pub extern "C" fn wasm_set_error(v: u32) {
        let mut error = ERROR.lock().expect("cannot be poisoned");
        *error = v;
    }

    fn get_error(register: u32, module: u32) -> Result<(), Error> {
        let error = *ERROR.lock().expect("cannot be poisoned");
        if error == NO_ERROR {
            Ok(())
        } else {
            Err(Box::new(crate::InterfaceError {
                register,
                module,
                error_type: match error.try_into() {
                    Ok(v) => v,
                    Err(_) => panic!("Could not convert error code"),
                },
            }))
        }
    }

    fn clear_error() {
        wasm_set_error(NO_ERROR);
    }

    fn read_from_buffer_u32(count: u32) -> Vec<u32> {
        let mut res = vec![];
        let mut buffer = BUFFER.lock().expect("cannot be poisoned");
        for i in 0..count {
            let mut b = [0u8; 4];
            b[..].copy_from_slice(&buffer[4 * i as usize..(i as usize + 1) * 4]);
            res.push(u32::from_le_bytes(b));
        }
        res
    }

    fn read_from_buffer_string(length: u32) -> String {
        let mut buffer = BUFFER.lock().expect("cannot be poisoned");
        match String::from_utf8(buffer[0..length as usize].to_vec()) {
            Ok(v) => v,
            Err(_) => {
                wasm_set_error(crate::ErrorType::WrongType as u32);
                "<non utf8 string>".to_owned()
            }
        }
    }

    pub struct StaticInterface;

    impl crate::Interface for StaticInterface {
        /// Retrieve the list of module ids that are available.
        fn modules(&self) -> Result<Vec<u32>, Error> {
            clear_error();
            let count = unsafe { wasm_interface_modules() };
            get_error(0, 0)?;
            Ok(read_from_buffer_u32(count))
        }

        /// Retrieve the name of a particular module.
        fn module_name(&self, module: u32) -> Result<String, Error> {
            clear_error();
            let string_length = unsafe { wasm_interface_module_name(module) };
            get_error(module, 0)?;
            let res = read_from_buffer_string(string_length);
            get_error(module, 0)?;
            Ok(res)
        }

        /// Return the available register ids in a particular module.
        fn registers(&self, module: u32) -> Result<Vec<u32>, Error> {
            clear_error();
            let count = unsafe { wasm_interface_registers(module) };
            get_error(module, 0)?;
            Ok(read_from_buffer_u32(count))
        }

        /// Retrieve a register name.
        fn register_name(&self, module: u32, register: u32) -> Result<String, Error> {
            clear_error();
            let string_length = unsafe { wasm_interface_register_name(module, register) };
            get_error(module, register)?;
            let res = read_from_buffer_string(string_length);
            get_error(module, register)?;
            Ok(res)
        }

        /// Retrieve a register type.
        fn register_type(&self, module: u32, register: u32) -> Result<RegisterType, Error> {
            clear_error();
            let register_type = unsafe { wasm_interface_register_type(module, register) };
            get_error(module, register)?;
            match register_type.try_into() {
                Ok(v) => Ok(v),
                Err(_) => panic!("Could not convert register type"),
            }
        }

        /// Get an i32 register.
        fn get_i32(&self, module: u32, register: u32) -> Result<i32, Error> {
            clear_error();
            let result = unsafe { wasm_interface_get_i32(module, register) };
            get_error(module, register)?;
            Ok(result)
        }

        /// Set an i32 register.
        fn set_i32(&mut self, module: u32, register: u32, value: i32) -> Result<i32, Error> {
            clear_error();
            let result = unsafe { wasm_interface_set_i32(module, register, value) };
            get_error(module, register)?;
            Ok(result)
        }

        /// Get an f32 register.
        fn get_f32(&self, module: u32, register: u32) -> Result<f32, Error> {
            clear_error();
            let result = unsafe { wasm_interface_get_f32(module, register) };
            get_error(module, register)?;
            Ok(result)
        }

        /// Set an f32 register.
        fn set_f32(&mut self, module: u32, register: u32, value: f32) -> Result<f32, Error> {
            clear_error();
            let result = unsafe { wasm_interface_set_f32(module, register, value) };
            get_error(module, register)?;
            Ok(result)
        }

        /// Get the length required to read a byte register.
        fn get_bytes_len(&self, module: u32, register: u32) -> Result<usize, Error> {
            clear_error();
            let result = unsafe { wasm_interface_get_bytes_len(module, register) };
            get_error(module, register)?;
            Ok(result as usize)
        }

        /// Get the actual bytes of a byte register, returns the number of bytes written.
        fn get_bytes(
            &self,
            module: u32,
            register: u32,
            destination: &mut [u8],
        ) -> Result<usize, Error> {
            clear_error();
            let result = unsafe {
                wasm_interface_get_bytes(
                    module,
                    register,
                    destination.as_ptr() as u32,
                    destination.len() as u32,
                )
            };
            get_error(module, register)?;
            Ok(result as usize)
        }

        /// Set a byte register.
        fn set_bytes(&mut self, module: u32, register: u32, values: &[u8]) -> Result<(), Error> {
            clear_error();
            unsafe {
                wasm_interface_set_bytes(
                    module,
                    register,
                    values.as_ptr() as u32,
                    values.len() as u32,
                )
            };
            get_error(module, register)?;
            Ok(())
        }
    }
}

mod logging {
    extern "C" {
        fn wasm_log_record(p: *const u8, len: u32);
    }
    use log::{Level, LevelFilter, Metadata, Record};
    static MY_LOGGER: MyLogger = MyLogger;
    struct MyLogger;

    impl log::Log for MyLogger {
        fn enabled(&self, metadata: &Metadata) -> bool {
            metadata.level() <= Level::Info
        }

        fn log(&self, record: &Record) {
            if self.enabled(record.metadata()) {
                let z = format!("{} - {}", record.level(), record.args()).to_string();
                unsafe {
                    wasm_log_record(&z.as_bytes()[0] as *const u8, z.len() as u32);
                }
            }
        }
        fn flush(&self) {}
    }

    pub fn setup() {
        log::set_logger(&MY_LOGGER).unwrap();
        log::set_max_level(LevelFilter::Info);
    }
}
