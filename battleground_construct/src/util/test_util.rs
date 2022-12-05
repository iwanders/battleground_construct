/// Asserts if a exceeds b by more than error.
pub fn approx_equal<T: cgmath::BaseFloat + std::fmt::Display>(a: T, b: T, max_error: T)
where
    T: std::ops::Sub<T>,
{
    let delta = (a - b).abs();
    if delta > max_error {
        panic!("a: {a:?}, b: {b:?},  delta was {delta}, this exceeded allowed {max_error}.");
    }
}
