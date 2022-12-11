use battleground_construct::components::vehicle_interface::RegisterInterface;
use battleground_vehicle_control::{Interface, VehicleControl};

use wasmtime::{Caller, Engine, Extern, Instance, Linker, Module, Store, TypedFunc};

struct State {
    register_interface: RegisterInterface,
}

struct VehicleControlWasm {
    engine: Engine,
    instance: Instance,
    store: Store<State>,
}
// Is this ok..?
unsafe impl std::marker::Send for VehicleControlWasm {}

impl VehicleControlWasm {
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let serialized_module_file =
            "../target/wasm32-unknown-unknown/release/example_driving_ai.wasm";
        let mut config = wasmtime::Config::default();
        config.consume_fuel(true);

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

        linker.func_wrap(
            "env",
            "wasm_interface_modules",
            |mut caller: Caller<'_, State>| -> u32 {
                let wasm_transmission_buffer = get_wasm_transmission_buffer(&mut caller);
                let wasm_set_error = get_wasm_set_error(&mut caller);

                let interfaces = caller.data().register_interface.modules();
                match interfaces {
                    Ok(v) => {
                        let count = v.len();
                        let data_width = count * std::mem::size_of::<u32>();
                        let p = wasm_transmission_buffer
                            .call(&mut caller, data_width as u32)
                            .expect("allocating memory failed");
                        println!("p: {p:x}");

                        let mem = caller
                            .get_export("memory")
                            .expect("memory should exist")
                            .into_memory()
                            .expect("was not memory");
                        let (bytes, storage) = mem.data_and_store_mut(&mut caller);
                        println!("Bytes length: {}", bytes.len());
                        // Now, we can write...
                        for (i, module) in v.iter().enumerate() {
                            let mut dest = &mut bytes[(i * std::mem::size_of::<u32>())
                                ..(i + 1) * std::mem::size_of::<u32>()];
                            let content = module.to_le_bytes();
                            dest.copy_from_slice(&content);
                        }
                        count as u32
                    }
                    Err(e) => {
                        wasm_set_error.call(caller, e.error_type as u32);
                        0
                    }
                }
                /**/
            },
        )?;

        linker.func_wrap(
            "env",
            "wasm_log_record",
            |mut caller: Caller<'_, State>, ptr: i32, len: i32| {
                // println!("Pointer: caller: {caller:?}");
                println!("Pointer: {ptr:?}, len: {len}");

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
                println!("data: {data:?}");
                let string = match data {
                    Some(data) => match std::str::from_utf8(data) {
                        Ok(s) => s,
                        Err(_) => "buhu",
                    },
                    None => "out of bounds",
                };
                println!("string: {string:?}");
            },
        );

        let module = Module::from_file(&engine, serialized_module_file)?;
        let state_object = State {
            register_interface: RegisterInterface::new(),
        };
        let mut store = Store::new(&engine, state_object);
        store.add_fuel(10000000)?;
        let instance = linker.instantiate(&mut store, &module)?;

        let mut exports = module.exports();
        for exp in exports {
            println!("exp: {}", exp.name());
            // match foo.ty() {
            // ExternType::Func(_) => { /* ... */ }
            // _ => panic!("unexpected export type!"),
            // }
        }

        // Obtain the setup function and call it.
        let wasm_setup = instance
            .get_func(&mut store, "wasm_setup")
            .expect("`wasm_setup` was not an exported function");
        let wasm_setup_fun = wasm_setup.typed::<(), (), _>(&store)?;
        wasm_setup_fun.call(&mut store, ())?;

        Ok(VehicleControlWasm {
            engine,
            store,
            instance,
        })
    }
}

impl VehicleControl for VehicleControlWasm {
    fn update(&mut self, interface: &mut dyn Interface) {
        // Clunky, but ah well... interface can't outlive this scope, so setting functions here that
        // use it doesn't work. Instead, copy the interface's state completely.

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

        wasm_controller_update
            .call(&mut self.store, ())
            .expect("shouldnt fail");

        // Write back the register interface.
        self.store
            .data()
            .register_interface
            .write_interface(interface)
            .expect("shouldnt fail");
    }
}

#[no_mangle]
pub fn create_ai() -> Box<dyn VehicleControl> {
    Box::new(VehicleControlWasm::new().unwrap())
}
