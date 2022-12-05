macro_rules! approx_equal {
    ($a:expr, $b: expr, $max_error:expr) => {
        let delta = ($a - $b).abs();
        if delta > $max_error {
            panic!(
                "a: {a:?}, b: {b:?},  delta was {delta}, this exceeded allowed {max_error}.",
                a = $a,
                b = $b,
                max_error = $max_error
            );
        }
    };
}

// https://stackoverflow.com/a/31749071  export the macro local to this file into the module.
pub(crate) use approx_equal;
