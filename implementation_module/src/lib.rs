
use once_cell::sync::Lazy;
use std::sync::Mutex;

use log::{info, warn};


extern "C" {
    pub fn foo();
}

pub fn safe_foo() {
    unsafe {foo()};
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

    values[0] + values[values.len() - 1] +  values[values.len() - 2] 
}

#[no_mangle]
pub extern "C" fn call_foo(){
    unsafe{foo()}
}

#[derive(Default, Debug)]
struct MyState{
    v: u32,
}


static STATE: Lazy<Mutex<MyState>> = Lazy::new(|| {
    Mutex::new(MyState::default())
});


#[no_mangle]
pub extern "C" fn set_state(v: u32){
    STATE.lock().unwrap().v = v;
}

#[no_mangle]
pub extern "C" fn get_state() -> u32{
    STATE.lock().unwrap().v
}


trait Handler: Send + Sync{
    fn update(&mut self);
}


static HANDLER: Mutex<Option<Box<dyn Handler>>> = Mutex::new(None);

fn register_handler(v: Box<dyn Handler>)
{
    *HANDLER.lock().unwrap() = Some(v);
}


struct MyHandler{
}

impl Handler for MyHandler {
    fn update(&mut self){
        safe_foo();
    }
}

#[no_mangle]
pub extern "C" 
fn setup_handler(){
    register_handler(Box::new(MyHandler{}));
}

#[no_mangle]
pub extern "C" 
fn call_handler() {
    HANDLER.lock().unwrap().as_mut().unwrap().update();
}


// Log setup.
extern "C" {
    // Ehh, how to emit a large chunk? :|
    pub fn log_record(p: * const u8, len: u32);
}

use log::{Record, Level, Metadata, LevelFilter};
static MY_LOGGER: MyLogger = MyLogger;
struct MyLogger;

impl log::Log for MyLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            // let foo = "mystring";
            // let foo = [1, 2, 3, 4, 5, 6, 7, 8u8];
            let foo = vec![1u8, 33, 3, 4, 5, 6, 7, 8];
            // let foo = [1, 2, 3, 4, 5, 6, 7, 8u8];
            unsafe {
                log_record(&foo[0] as *const u8, foo.len() as u32);
            }
            println!("{} - {}", record.level(), record.args());
        }
    }
    fn flush(&self) {}
}

#[no_mangle]
pub extern "C" 
fn log_setup() {
    log::set_logger(&MY_LOGGER).unwrap();
    log::set_max_level(LevelFilter::Info);
}

#[no_mangle]
pub extern "C" 
fn log_test(){
    info!("test info log {}", 3);
}
