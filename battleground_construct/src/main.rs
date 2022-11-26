use battleground_construct::Construct;
use battleground_construct::vehicle_control;

struct DummyInterface {}
impl vehicle_control::Interface for DummyInterface {
fn registers(&self) -> usize { todo!() }
fn get_u32(&self, _: usize) -> Result<u32, Box<(dyn std::error::Error + 'static)>> { todo!() }
fn set_u32(&mut self, _: usize, _: u32) -> Result<u32, Box<(dyn std::error::Error + 'static)>> { todo!() }

}

fn main() -> Result<(),Box<dyn std::error::Error>> {

    let lib = unsafe {libloading::Library::new("../target/release/libexample_driving_ai.so")?};
    let mut res = unsafe {
        let func: libloading::Symbol<vehicle_control::ControllerSpawn> = lib.get(b"create_ai")?;
        let foo = func();
        println!("invocation succeeded");
        foo
    };

    println!("res: exists after lib went out of scope.");

    // let mut construct = Construct::new();
    // let max_time = 200.0;
    // while construct.elapsed_as_f64() < max_time {
        // construct.update();
    // }
    let mut interface = DummyInterface{};
    res.update(&mut interface);

    

    Ok(())
}
