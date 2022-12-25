use battleground_unit_control::register_interface::RegisterInterface;
use battleground_unit_control::{Interface, InterfaceError, UnitControl};

use wasmtime::{Caller, Engine, Extern, Instance, Linker, Module, Store, TypedFunc};

struct State {
    register_interface: RegisterInterface,
}

pub struct UnitControlWasm {
    // engine: Engine,
    instance: Instance,
    store: Store<State>,
}
// Is this ok..?
unsafe impl std::marker::Send for UnitControlWasm {}

impl UnitControlWasm {
    pub fn new(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let serialized_module_file = path;
        let mut config = wasmtime::Config::default();
        config.consume_fuel(true);
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

        let module = Module::from_file(&engine, serialized_module_file)?;
        let state_object = State {
            register_interface: RegisterInterface::new(),
        };
        let mut store = Store::new(&engine, state_object);
        store.add_fuel(10000000)?;
        let instance = linker.instantiate(&mut store, &module)?;

        let exports = module.exports();
        for exp in exports {
            println!("exp: {}", exp.name());
        }

        // Obtain the setup function and call it.
        let wasm_setup = instance
            .get_func(&mut store, "wasm_setup")
            .expect("`wasm_setup` was not an exported function");
        let wasm_setup_fun = wasm_setup.typed::<(), (), _>(&store)?;
        wasm_setup_fun.call(&mut store, ())?;

        Ok(UnitControlWasm {
            // engine,
            store,
            instance,
        })
    }
}

impl UnitControl for UnitControlWasm {
    fn update(&mut self, interface: &mut dyn Interface) {
        // Clunky, but ah well... interface can't outlive this scope, so setting functions here that
        // use it doesn't work. Instead, copy the interface's state completely.

        self.store.add_fuel(100000000).expect("");

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
            .typed::<(), (), _>(&self.store)
            .expect("should be correct signature");

        let update_res = wasm_controller_update.call(&mut self.store, ());
        match update_res {
            Ok(_) => {}
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
    }
}

#[no_mangle]
pub fn create_ai() -> Box<dyn UnitControl> {
    Box::new(
        UnitControlWasm::new("../target/wasm32-unknown-unknown/release/unit_control_example.wasm")
            .unwrap(),
    )
}
