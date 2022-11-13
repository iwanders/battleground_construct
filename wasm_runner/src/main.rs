
use wasmer::{imports, Instance, Module, Store, TypedFunction, Value, EngineBuilder, Function, FunctionEnv, FunctionEnvMut, Memory, WasmPtr, MemoryType, ValueType};
use wasmer_compiler_cranelift::Cranelift;

use std::sync::Arc;
use wasmer::wasmparser::Operator;

use wasmer::CompilerConfig;

use wasmer_middlewares::{
    metering::{get_remaining_points, set_remaining_points, MeteringPoints},
    Metering,
};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // cost function from https://github.com/wasmerio/wasmer/blob/v3.0.0-rc.2/examples/metering.rs

    let cost_function = |operator: &Operator| -> u64 {
        match operator {
            Operator::LocalGet { .. } | Operator::I32Const { .. } => 1,
            Operator::I32Add { .. } => 2,
            _ => 0,
        }
    };

    let metering = Arc::new(Metering::new(10000000, cost_function));
    let mut compiler_config = Cranelift::default();
    compiler_config.push_middleware(metering);
    let mut store = Store::new(EngineBuilder::new(compiler_config));

    // Function to be provided from this side.
    struct MyEnv;
    let env = FunctionEnv::new(&mut store, MyEnv {});
    fn foo(_env: FunctionEnvMut<MyEnv>) {
        println!("Foo called");
    }
    let foo_typed = Function::new_typed_with_env(&mut store, &env, foo);



    // https://github.com/wasmerio/wasmer/issues/3274
    let my_memory = Memory::new(&mut store, MemoryType::new(1, None, false)).unwrap();
    let env_with_mem = FunctionEnv::new(&mut store, EnvWithMemory {memory: my_memory});
    struct EnvWithMemory {
        memory: Memory,
    };
    fn print_string(_env: FunctionEnvMut<EnvWithMemory>, pos: u32, len: u32) {
    }
    let print_string_typed = Function::new_typed_with_env(&mut store, &env_with_mem, print_string);




    //    pub fn log_record(p: * const u8 , len: u64);
    
    fn log_record(env: FunctionEnvMut<EnvWithMemory>, ptr: WasmPtr<[u8; 8]>, len: u32){
        println!("Pointer: {ptr:?}, len: {len}");
        let mem = env.data().memory.view(&env);
        let outbuf: WasmPtr<u8> = unsafe{ std::mem::transmute(ptr) };
        println!("outbuf: {outbuf:?}");
        let mut v = [0u8; 8];
        let s = outbuf.slice(&mem, 8).unwrap();
        s.write_slice(&mut v[..]);
        println!("outbuf: {s:?}");
        println!("v: {v:?}");    
    }
    let log_record_typed = Function::new_typed_with_env(&mut store, &env_with_mem, log_record);


    let import_object = imports! {
        "env" => {
            "foo" => foo_typed,
            "print_string" => print_string_typed,
            "log_record" => log_record_typed,
            // "get_memory_typed" => get_memory_typed,
        }
    };

    // Load the wasm module.
    let serialized_module_file = "../implementation_module/target/wasm32-unknown-unknown/debug/implementation_module.wasm";
    let module = Module::from_file(&store, serialized_module_file)?;
    println!("Module: {module:?}");
    for export_ in module.exports() {
        println!("{:?}", export_.ty());
    }


    println!("Instantiating module...");
    let instance = Instance::new(&mut store, &module, &import_object)?;


    println!("points: {:?}", get_remaining_points(&mut store, &instance));

    // Test sum.
    {
        // Get the function.
        let sum = instance.exports.get_function("sum")?;

        println!("Calling `sum` function...");

        let args = [Value::I32(1), Value::I32(5)];
        let result = sum.call(&mut store, &args)?;
        println!("points: {:?}", get_remaining_points(&mut store, &instance));


        println!("Results: {:?}", result);
        assert_eq!(result.to_vec(), vec![Value::I32(1 + 5)]);

        // Call it as a typed function.
        let sum_typed: TypedFunction<(i32, i32), i32> = sum.typed(&mut store)?;

        println!("Calling `sum` function (natively)...");
        let result = sum_typed.call(&mut store, 1, 5)?;

        println!("Results: {:?}", result);
        assert_eq!(result, 6);
        println!("points: {:?}", get_remaining_points(&mut store, &instance));

    }

    // test foo
    {
        let call_foo = instance.exports.get_function("call_foo")?;
        let foo_typed: TypedFunction<(), ()> = call_foo.typed(&mut store)?;
        let _res = foo_typed.call(&mut store)?;
    }

    // test alloc 
    {
        let sum_with_alloc = instance.exports.get_function("sum_with_alloc")?;
        let sum_with_alloc_typed: TypedFunction<u64, u64> = sum_with_alloc.typed(&mut store)?;
        let result = sum_with_alloc_typed.call(&mut store, 100)?;
        assert_eq!(result, 197);
    }

    // test opaque state 
    {
        let set_state = instance.exports.get_function("set_state")?;
        let get_state = instance.exports.get_function("get_state")?;
        let set_state_typed: TypedFunction<u32, ()> = set_state.typed(&mut store)?;
        let get_state_typed: TypedFunction<(), u32> = get_state.typed(&mut store)?;
        let _result = set_state_typed.call(&mut store, 100)?;
        assert_eq!(get_state_typed.call(&mut store)?, 100u32);
        let _result = set_state_typed.call(&mut store, 101)?;
        assert_eq!(get_state_typed.call(&mut store)?, 101u32);
    }

    // Try the handler
    {
        let setup_handler = instance.exports.get_function("setup_handler")?;
        let setup_handler_typed: TypedFunction<(), ()> = setup_handler.typed(&mut store)?;
        let _res = setup_handler_typed.call(&mut store)?;

        let call_handler = instance.exports.get_function("call_handler")?;
        let call_handler_typed: TypedFunction<(), ()> = call_handler.typed(&mut store)?;
        let _res = call_handler_typed.call(&mut store)?;
    }

    // Try the logger
    {
        let log_setup = instance.exports.get_function("log_setup")?;
        let log_setup_typed: TypedFunction<(), ()> = log_setup.typed(&mut store)?;
        let _res = log_setup_typed.call(&mut store)?;
        let log_test = instance.exports.get_function("log_test")?;
        let log_test_typed: TypedFunction<(), ()> = log_test.typed(&mut store)?;
        let _res = log_test_typed.call(&mut store)?;
    }

    Ok(())
}
