use battleground_unit_control::register_interface::RegisterInterface;
use battleground_unit_control::{Interface, InterfaceError, UnitControl};

use wasmtime::{Caller, Engine, Extern, Instance, Linker, Module, Store, TypedFunc};

/// Configuration struct for the wasm control unit.
#[derive(Clone, Debug)]
pub struct UnitControlWasmConfig {
    /// Path to the wasm file to load, it MUST provide the helpers from [`battleground_unit_control::wasm_interface`] side.
    pub wasm_path: std::path::PathBuf,

    /// The alloted fuel per update call, if this is exceeded an exception is raised.
    pub fuel_per_update: Option<u64>,

    /// Print the exports provided by the wasm module?
    pub print_exports: bool,

    /// The alloted fuel for the setup call, defaults to 100000000 if fuel_per_update is not None.
    pub fuel_for_setup: Option<u64>,
}

struct State {
    register_interface: RegisterInterface,
    control_update_error: String,
}

pub struct UnitControlWasm {
    // engine: Engine,
    instance: Instance,
    store: Store<State>,
    control_config: UnitControlWasmConfig,
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

        let engine = Engine::new(&config)?;
        let mut linker = Linker::<State>::new(&engine);

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
                println!("{string:?}");
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

        if !control_config.wasm_path.is_file() {
            return Err(format!(
                "module {} could not be found.",
                control_config.wasm_path.display()
            )
            .into());
        }

        let module = Module::from_file(&engine, control_config.wasm_path.clone())?;
        let state_object = State {
            register_interface: RegisterInterface::new(),
            control_update_error: Default::default(),
        };
        let mut store = Store::new(&engine, state_object);
        if using_fuel {
            // We cannot add inifinite fuel for setup, the fuel is calculated based on lifetime
            // consumed vs lifetime added, it is not a proper level.
            let setup_fuel = control_config.fuel_for_setup.unwrap_or(100000000);
            store.add_fuel(setup_fuel)?;
        }

        let instance = linker.instantiate(&mut store, &module)?;

        let exports = module.exports();

        if control_config.print_exports {
            for exp in exports {
                println!("exp: {}", exp.name());
            }
        }

        // Obtain the setup function and call it.
        let wasm_setup = instance
            .get_func(&mut store, "wasm_setup")
            .expect("`wasm_setup` was not an exported function");
        let wasm_setup_fun = wasm_setup.typed::<(), (), _>(&store)?;
        wasm_setup_fun.call(&mut store, ())?;

        Ok(UnitControlWasm {
            control_config,
            store,
            instance,
        })
    }
}

impl UnitControl for UnitControlWasm {
    fn update(&mut self, interface: &mut dyn Interface) -> Result<(), Box<dyn std::error::Error>> {
        // Clunky, but ah well... interface can't outlive this scope, so setting functions here that
        // use it doesn't work. Instead, copy the interface's state completely.

        if let Some(v) = self.control_config.fuel_per_update {
            self.zero_fuel();
            // println!("V: {v}");
            self.store.add_fuel(v).expect("adding fuel failed");
            // let remaining = self.remaining_fuel();
            // println!("after addition: {remaining}");
        }

        // Copy all registers into the state.
        self.store
            .data_mut()
            .register_interface
            .read_interface(interface)
            .expect("shouldnt fail");

        // execute the controller.
        let wasm_controller_update = self
            .instance
            .get_func(&mut self.store, "wasm_controller_update")
            .expect("`wasm_controller_update` was not an exported function");
        let wasm_controller_update = wasm_controller_update
            .typed::<(), i64, _>(&self.store)
            .expect("should be correct signature");

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
            Err(v) => {
                println!("Something went wrong in update {v:?}");
                panic!();
            }
        }

        // Write back the register interface.
        self.store
            .data()
            .register_interface
            .write_interface(interface)
            .expect("shouldnt fail");

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
