
use once_cell::sync::Lazy;
use std::sync::Mutex;

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
    HANDLER.lock().unwrap().as_mut().unwrap().update();
    HANDLER.lock().unwrap().as_mut().unwrap().update()
}


