use once_cell::sync::Lazy;
use std::sync::Mutex;

use log::{info, warn};

extern "C" {
    pub fn foo();
}

pub fn safe_foo() {
    unsafe { foo() };
}

#[no_mangle]
pub extern "C" fn sum(v: i32, v1: i32) -> i32 {
    v + v1
}

#[no_mangle]
pub extern "C" fn sum_with_alloc(up_to: u64) -> u64 {
    let mut values: Vec<u64> = vec![];
    values.reserve(up_to as usize);
    for i in 0..up_to {
        values.push(i)
    }

    values[0] + values[values.len() - 1] + values[values.len() - 2]
}

#[no_mangle]
pub extern "C" fn call_foo() {
    unsafe { foo() }
}

#[derive(Default, Debug)]
struct MyState {
    v: u32,
}

static STATE: Lazy<Mutex<MyState>> = Lazy::new(|| Mutex::new(MyState::default()));

#[no_mangle]
pub extern "C" fn set_state(v: u32) {
    STATE.lock().unwrap().v = v;
}

#[no_mangle]
pub extern "C" fn get_state() -> u32 {
    STATE.lock().unwrap().v
}

trait Handler: Send + Sync {
    fn update(&mut self);
}

static HANDLER: Mutex<Option<Box<dyn Handler>>> = Mutex::new(None);

fn register_handler(v: Box<dyn Handler>) {
    *HANDLER.lock().unwrap() = Some(v);
}

struct MyHandler {}

impl Handler for MyHandler {
    fn update(&mut self) {
        safe_foo();
    }
}

#[no_mangle]
pub extern "C" fn setup_handler() {
    register_handler(Box::new(MyHandler {}));
}

#[no_mangle]
pub extern "C" fn call_handler() {
    HANDLER.lock().unwrap().as_mut().unwrap().update();
}

// Log setup.
extern "C" {
    // Send a pointer, other side inspects memory and mucks around with it.
    pub fn log_record(p: *const u8, len: u32);
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
                log_record(&z.as_bytes()[0] as *const u8, z.len() as u32);
            }
        }
    }
    fn flush(&self) {}
}

#[no_mangle]
pub extern "C" fn log_setup() {
    log::set_logger(&MY_LOGGER).unwrap();
    log::set_max_level(LevelFilter::Info);
}

#[no_mangle]
pub extern "C" fn log_test() {
    info!("test info log {}", 3);
}



static INPUT_BUFFER: Mutex<Vec<u8>> = Mutex::new(Vec::new());

#[no_mangle]
pub extern "C" 
fn prepare_input(len: u32) -> *mut u8 {
    let mut buffer = INPUT_BUFFER.lock().expect("cannot be poisoned");
    buffer.clear();
    buffer.resize(len as usize, 0);
    buffer.as_mut_ptr()
}

#[no_mangle]
pub extern "C" fn use_input(len: u32) {
    info!("Input buffer now holds: {:?}", &INPUT_BUFFER.lock().expect("cannot be poisoned")[..len as usize]);
}





