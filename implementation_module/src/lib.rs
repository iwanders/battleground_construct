
extern "C" {
    pub fn foo();
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
