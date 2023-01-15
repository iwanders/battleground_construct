use battleground_unit_control::register_interface::RegisterInterface;
use battleground_unit_control::{Interface, InterfaceError, UnitControl};
use std::time::SystemTime;

use wasmtime::{Caller, Engine, Extern, Instance, Linker, Module, Store, TypedFunc};

/// Configuration struct for the wasm control unit.
#[derive(Clone, Debug)]
pub struct UnitControlWasmConfig {
    /// Path to the wasm file to load, it MUST provide the helpers from [`battleground_unit_control::wasm_interface`] side.
    pub wasm_path: std::path::PathBuf,

    /// The alloted fuel per update call, if this is exceeded an exception is raised.
    pub fuel_per_update: Option<u64>,

    /// Print the exports provided by the wasm module?
    // pub print_exports: bool,

    /// The alloted fuel for the setup call, defaults to 100000000 if fuel_per_update is not None.
    pub fuel_for_setup: Option<u64>,

    /// Automatically reload the wasm file from disk if it has been modified. If it fails to load
    /// the old file remains in use.
    pub reload: bool,
}

struct State {
    register_interface: RegisterInterface,
    control_update_error: String,
}

pub struct UnitControlWasm {
    engine: Engine,
    instance: Instance,
    store: Store<State>,
    control_config: UnitControlWasmConfig,
    finished_module_setup: bool,
    modified_time: SystemTime,
}
// Is this ok..?
// unsafe impl std::marker::Send for UnitControlWasm {}

impl UnitControlWasm {
    pub fn zero_fuel(&mut self) {
        // Well.. https://github.com/bytecodealliance/wasmtime/pull/5220
        // we can call consume with zero, to get the current value, and then consume that.
        let remaining = self
            .store
            .consume_fuel(0)
            .expect("always valid on store with fuel");
        let v = self
            .store
            .consume_fuel(remaining)
            .expect("always valid on store with fuel");
        assert_eq!(v, 0);
    }

    pub fn remaining_fuel(&mut self) -> u64 {
        self.store
            .consume_fuel(0)
            .expect("always valid on store with fuel")
    }

    pub fn new_with_config(
        control_config: UnitControlWasmConfig,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let mut config = wasmtime::Config::default();

        let using_fuel = control_config.fuel_per_update.is_some();

        if using_fuel {
            config.consume_fuel(true);
        }
        // Always assume we have symbols, otherwise people will have a very bad day debugging.
        config.debug_info(true);
        // Always get line numbers from the debug symbols;
        // https://docs.rs/wasmtime/latest/wasmtime/struct.Config.html#method.wasm_backtrace_details
        config.wasm_backtrace_details(wasmtime::WasmBacktraceDetails::Enable);

        let engine = Engine::new(&config)?;

        let state_object = State {
            register_interface: RegisterInterface::new(),
            control_update_error: Default::default(),
        };
        let mut store = Store::new(&engine, state_object);

        let (instance, modified_time) =
            Self::load_file(&mut store, &engine, &control_config.wasm_path)?;

        Ok(UnitControlWasm {
            engine,
            finished_module_setup: false,
            control_config,
            store,
            instance,
            modified_time,
        })
    }

    fn load_file(
        store: &mut Store<State>,
        engine: &Engine,
        path: &std::path::Path,
    ) -> Result<(Instance, SystemTime), Box<dyn std::error::Error>> {
        let mut linker = Linker::<State>::new(engine);

        fn get_wasm_transmission_buffer(caller: &mut Caller<'_, State>) -> TypedFunc<u32, u32> {
            let wasm_transmission_buffer = caller
                .get_export("wasm_transmission_buffer")
                .expect("Wasm32 module must expose this")
                .into_func()
                .expect("`wasm_transmission_buffer` was not an exported function");
            wasm_transmission_buffer
                .typed::<u32, u32, _>(&caller)
                .expect("signature should match")
        }

        fn get_wasm_set_error(caller: &mut Caller<'_, State>) -> TypedFunc<u32, ()> {
            let wasm_set_error = caller
                .get_export("wasm_set_error")
                .expect("Wasm32 module must expose this")
                .into_func()
                .expect("`wasm_set_error` was not an exported function");
            wasm_set_error
                .typed::<u32, (), _>(&caller)
                .expect("signature should match")
        }

        fn send_vec_u32_result(
            mut caller: Caller<'_, State>,
            res: Result<Vec<u32>, Box<InterfaceError>>,
        ) -> usize {
            let wasm_transmission_buffer = get_wasm_transmission_buffer(&mut caller);
            let wasm_set_error = get_wasm_set_error(&mut caller);
            match res {
                Ok(v) => {
                    let count = v.len();
                    let data_width = count * std::mem::size_of::<u32>();
                    let p = wasm_transmission_buffer
                        .call(&mut caller, data_width as u32)
                        .expect("allocating memory failed");

                    let mem = caller
                        .get_export("memory")
                        .expect("memory should exist")
                        .into_memory()
                        .expect("was not memory");
                    let (bytes, _storage) = mem.data_and_store_mut(&mut caller);

                    // Now, we can write...
                    for (i, module) in v.iter().enumerate() {
                        let dest = &mut bytes[(p as usize + (i * std::mem::size_of::<u32>()))
                            ..(p as usize + (i + 1) * std::mem::size_of::<u32>())];
                        let content = module.to_le_bytes();
                        dest.copy_from_slice(&content);
                    }
                    count
                }
                Err(e) => {
                    wasm_set_error
                        .call(caller, e.error_type as u32)
                        .expect("could not set error");
                    0
                }
            }
        }

        fn send_string_result(
            mut caller: Caller<'_, State>,
            res: Result<String, Box<InterfaceError>>,
        ) -> usize {
            let wasm_transmission_buffer = get_wasm_transmission_buffer(&mut caller);
            let wasm_set_error = get_wasm_set_error(&mut caller);
            match res {
                Ok(v) => {
                    let data = v.into_bytes();
                    let data_width = data.len();
                    let p = wasm_transmission_buffer
                        .call(&mut caller, data_width as u32)
                        .expect("allocating memory failed");

                    let mem = caller
                        .get_export("memory")
                        .expect("memory should exist")
                        .into_memory()
                        .expect("was not memory");
                    let (bytes, _storage) = mem.data_and_store_mut(&mut caller);

                    bytes[p as usize..(p as usize + data_width)].copy_from_slice(&data);
                    data_width
                }
                Err(e) => {
                    wasm_set_error
                        .call(caller, e.error_type as u32)
                        .expect("could not set error");
                    0
                }
            }
        }

        fn send_pod_result<C>(
            mut caller: Caller<'_, State>,
            res: Result<C, Box<InterfaceError>>,
            failed: C,
        ) -> C {
            let wasm_set_error = get_wasm_set_error(&mut caller);
            match res {
                Ok(v) => v,
                Err(e) => {
                    wasm_set_error
                        .call(caller, e.error_type as u32)
                        .expect("could not set error");
                    failed
                }
            }
        }

        linker.func_wrap(
            "env",
            "wasm_interface_modules",
            |caller: Caller<'_, State>| -> u32 {
                let res = caller.data().register_interface.modules();
                send_vec_u32_result(caller, res) as u32
            },
        )?;
        linker.func_wrap(
            "env",
            "wasm_interface_registers",
            |caller: Caller<'_, State>, module: u32| -> u32 {
                let res = caller.data().register_interface.registers(module);
                send_vec_u32_result(caller, res) as u32
            },
        )?;
        linker.func_wrap(
            "env",
            "wasm_interface_module_name",
            |caller: Caller<'_, State>, module: u32| -> u32 {
                let res = caller.data().register_interface.module_name(module);
                send_string_result(caller, res) as u32
            },
        )?;

        linker.func_wrap(
            "env",
            "wasm_interface_register_name",
            |caller: Caller<'_, State>, module: u32, register: u32| -> u32 {
                let res = caller
                    .data()
                    .register_interface
                    .register_name(module, register);
                send_string_result(caller, res) as u32
            },
        )?;

        linker.func_wrap(
            "env",
            "wasm_interface_register_type",
            |mut caller: Caller<'_, State>, module: u32, register: u32| -> u32 {
                let wasm_set_error = get_wasm_set_error(&mut caller);
                let res = caller
                    .data()
                    .register_interface
                    .register_type(module, register);
                match res {
                    Ok(v) => v as u32,
                    Err(e) => {
                        wasm_set_error
                            .call(caller, e.error_type as u32)
                            .expect("could not set error");
                        0
                    }
                }
            },
        )?;

        linker.func_wrap(
            "env",
            "wasm_interface_get_i32",
            |caller: Caller<'_, State>, module: u32, register: u32| -> i32 {
                let res = caller.data().register_interface.get_i32(module, register);
                send_pod_result(caller, res, 0)
            },
        )?;

        linker.func_wrap(
            "env",
            "wasm_interface_get_f32",
            |caller: Caller<'_, State>, module: u32, register: u32| -> f32 {
                let res = caller.data().register_interface.get_f32(module, register);
                send_pod_result(caller, res, 0.0)
            },
        )?;
        linker.func_wrap(
            "env",
            "wasm_interface_set_i32",
            |mut caller: Caller<'_, State>, module: u32, register: u32, value: i32| -> i32 {
                let res = caller
                    .data_mut()
                    .register_interface
                    .set_i32(module, register, value);
                send_pod_result(caller, res, 0)
            },
        )?;
        linker.func_wrap(
            "env",
            "wasm_interface_set_f32",
            |mut caller: Caller<'_, State>, module: u32, register: u32, value: f32| -> f32 {
                let res = caller
                    .data_mut()
                    .register_interface
                    .set_f32(module, register, value);
                send_pod_result(caller, res, 0.0)
            },
        )?;

        linker.func_wrap(
            "env",
            "wasm_interface_get_bytes_len",
            |mut caller: Caller<'_, State>, module: u32, register: u32| -> u32 {
                let res = caller
                    .data_mut()
                    .register_interface
                    .get_bytes_len(module, register);
                let res = match res {
                    Ok(v) => Ok(v as u32),
                    Err(e) => Err(e),
                };
                send_pod_result(caller, res, 0)
            },
        )?;
        linker.func_wrap(
            "env",
            "wasm_interface_get_bytes",
            |mut caller: Caller<'_, State>,
             module: u32,
             register: u32,
             dest: u32,
             len: u32|
             -> u32 {
                let mut tmp = vec![0; len as usize];

                let res = caller
                    .data()
                    .register_interface
                    .get_bytes(module, register, &mut tmp);

                let mem = caller
                    .get_export("memory")
                    .expect("memory should exist")
                    .into_memory()
                    .expect("was not memory");
                let (bytes, _storage) = mem.data_and_store_mut(&mut caller);

                let res = match res {
                    Ok(v) => {
                        bytes[dest as usize..(dest as usize + len as usize)].copy_from_slice(&tmp);
                        Ok(v as u32)
                    }
                    Err(e) => Err(e),
                };
                send_pod_result(caller, res, 0)
            },
        )?;
        linker.func_wrap(
            "env",
            "wasm_interface_set_bytes",
            |mut caller: Caller<'_, State>,
             module: u32,
             register: u32,
             src: u32,
             len: u32|
             -> u32 {
                let mut tmp = vec![0; len as usize];
                {
                    let mem = caller
                        .get_export("memory")
                        .expect("memory should exist")
                        .into_memory()
                        .expect("was not memory");
                    let (bytes, _storage) = mem.data_and_store_mut(&mut caller);
                    tmp.copy_from_slice(&bytes[src as usize..(src as usize + len as usize)]);
                }

                let res = caller
                    .data_mut()
                    .register_interface
                    .set_bytes(module, register, &tmp);

                let res = match res {
                    Ok(_) => Ok(0),
                    Err(e) => Err(e),
                };
                send_pod_result(caller, res, 0)
            },
        )?;

        linker.func_wrap(
            "env",
            "wasm_log_record",
            |mut caller: Caller<'_, State>, ptr: i32, len: i32| {
                let mem = match caller.get_export("memory") {
                    Some(Extern::Memory(mem)) => mem,
                    _ => {
                        println!("failed to find host memory");
                        return;
                    }
                };
                let data = mem
                    .data(&caller)
                    .get(ptr as u32 as usize..)
                    .and_then(|arr| arr.get(..len as u32 as usize));
                // println!("data: {data:?}");
                let string = match data {
                    Some(data) => std::str::from_utf8(data).unwrap_or("<non utf8 string>"),
                    None => "out of bounds",
                };
                // we definitely want something better here...
                println!("unit_control_wasm: {string:?}");
            },
        )?;

        linker.func_wrap(
            "env",
            "wasm_update_error",
            |mut caller: Caller<'_, State>, ptr: i32, len: i32| {
                let mem = match caller.get_export("memory") {
                    Some(Extern::Memory(mem)) => mem,
                    _ => {
                        println!("failed to find host memory");
                        return;
                    }
                };
                let data = mem
                    .data(&caller)
                    .get(ptr as u32 as usize..)
                    .and_then(|arr| arr.get(..len as u32 as usize));
                let err_string = match data {
                    Some(data) => std::str::from_utf8(data).unwrap_or("<non utf8 string>"),
                    None => "out of bounds",
                };
                caller.data_mut().control_update_error = err_string.to_owned();
            },
        )?;

        if !path.is_file() {
            return Err(format!("module {} could not be found.", path.display()).into());
        }

        let module = Module::from_file(engine, path)?;
        let instance = linker.instantiate(store, &module)?;

        let metadata = std::fs::metadata(path)?;
        let modification = metadata.modified()?;

        /*
        let exports = module.exports();

        if control_config.print_exports {
            for exp in exports {
                println!("exp: {}", exp.name());
            }
        }
        */

        Ok((instance, modification))
    }

    pub fn attempt_reload(&mut self) -> Result<bool, Box<dyn std::error::Error>> {
        // Try the reload.

        let metadata = std::fs::metadata(&self.control_config.wasm_path)?;
        let modification = metadata.modified()?;
        if modification > self.modified_time {
            let (instance, modified_time) = Self::load_file(
                &mut self.store,
                &self.engine,
                &self.control_config.wasm_path,
            )?;
            self.instance = instance;
            self.modified_time = modified_time;
            self.finished_module_setup = false; // ensure setup runs for the wasm interface.
            return Ok(true);
        }

        Ok(false)
    }
}

impl UnitControl for UnitControlWasm {
    fn update(&mut self, interface: &mut dyn Interface) -> Result<(), Box<dyn std::error::Error>> {
        // check if we need to swap the controller.
        if self.control_config.reload {
            match self.attempt_reload() {
                Ok(did_reload) => {
                    if did_reload {
                        println!("Reloaded {}", self.control_config.wasm_path.display())
                    }
                }
                Err(e) => println!(
                    "Failed reloading {}: {e:?}",
                    self.control_config.wasm_path.display()
                ),
            }
        }

        // First cycle? If so, do the setup.
        if !self.finished_module_setup {
            if self.control_config.fuel_per_update.is_some() {
                // We cannot add infinite fuel for setup, the fuel is calculated based on lifetime
                // consumed vs lifetime added, it is not a proper level.
                let setup_fuel = self.control_config.fuel_for_setup.unwrap_or(100000000);
                self.store.add_fuel(setup_fuel)?;
            }

            // Obtain the setup function and call it.
            let wasm_setup = self
                .instance
                .get_func(&mut self.store, "wasm_setup")
                .ok_or_else(|| {
                    Box::<dyn std::error::Error>::from("function wasm_setup not found in module")
                })?;
            let wasm_setup_fun = wasm_setup.typed::<(), (), _>(&self.store)?;
            wasm_setup_fun.call(&mut self.store, ())?;

            self.finished_module_setup = true;
        }

        if let Some(v) = self.control_config.fuel_per_update {
            self.zero_fuel();
            self.store.add_fuel(v).expect("adding fuel failed");
        }

        // Clunky, but ah well... interface can't outlive this scope, so setting functions here that
        // use it doesn't work. Instead, copy the interface's state completely.

        // Copy all registers into the state.
        self.store
            .data_mut()
            .register_interface
            .read_interface(interface)?;

        // execute the controller.
        let wasm_controller_update = self
            .instance
            .get_func(&mut self.store, "wasm_controller_update")
            .ok_or_else(|| {
                Box::<dyn std::error::Error>::from(
                    "function wasm_controller_update not found in module",
                )
            })?;

        let wasm_controller_update = wasm_controller_update.typed::<(), i64, _>(&self.store)?;
        let update_res = wasm_controller_update.call(&mut self.store, ());

        // Check the return code.
        match update_res {
            Ok(v) => {
                // Numbers match wasm interface.
                const UPDATE_OK: i64 = 0;
                match v {
                    UPDATE_OK => {
                        // do nothing, fall through, perform the register update and return ok.
                    }
                    _ => {
                        // the string in the store will be populated, convert this to an error and
                        // bubble up.
                        return Err(self.store.data().control_update_error.as_str().into());
                    }
                }
            }
            Err(e) => {
                // See https://docs.rs/wasmtime/latest/wasmtime/struct.Func.html#method.call
                println!("something went wrong in update: {e:?}");
                if e.is::<wasmtime::Trap>() {
                    let trap = e.downcast::<wasmtime::Trap>()?;
                    return Err(Box::<dyn std::error::Error>::from(
                        format!("trap: {:?}", trap).as_str(),
                    ));
                } else if e.is::<wasmtime::WasmBacktrace>() {
                    let bt = e.downcast::<wasmtime::WasmBacktrace>()?;
                    return Err(Box::<dyn std::error::Error>::from(
                        format!("bt: {:?}", bt).as_str(),
                    ));
                } else {
                    return Err(Box::<dyn std::error::Error>::from(
                        format!("{:?}", e).as_str(),
                    ));
                }
            }
        }

        // Write back the register interface.
        self.store
            .data()
            .register_interface
            .write_interface(interface)?;

        Ok(())
    }
}
/*
#[no_mangle]
pub fn create_ai() -> Box<dyn UnitControl> {
    Box::new(
        UnitControlWasm::new("../target/wasm32-unknown-unknown/release/unit_control_example.wasm")
            .unwrap(),
    )
}
*/
