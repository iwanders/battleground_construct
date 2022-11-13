
extern "C" {
    pub fn foo();
}



#[no_mangle]
pub extern "C" fn sum(v: i32, v1: i32) -> i32 {
    let mut zzz = vec![3u32];

    let f = v as f32;
    zzz.resize(100, 5);
    unsafe {foo()};
    v + v1 + 3// + (f.sin() * 100.0) as i32
}
