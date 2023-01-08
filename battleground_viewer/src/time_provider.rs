#![allow(dead_code, unused_imports)]
// https://github.com/rust-lang/rust/issues/48564#issuecomment-505114709

use std::convert::TryInto;
use std::ops::{Add, AddAssign, Sub, SubAssign};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

pub use std::time::*;

#[cfg(not(target_arch = "wasm32"))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Instant(std::time::Instant);
#[cfg(not(target_arch = "wasm32"))]
impl Instant {
    pub fn now() -> Self {
        Self(std::time::Instant::now())
    }
    pub fn duration_since(&self, earlier: Instant) -> Duration {
        self.0.duration_since(earlier.0)
    }
    pub fn elapsed(&self) -> Duration {
        self.0.elapsed()
    }
    pub fn checked_add(&self, duration: Duration) -> Option<Self> {
        self.0.checked_add(duration).map(Self)
    }
    pub fn checked_sub(&self, duration: Duration) -> Option<Self> {
        self.0.checked_sub(duration).map(Self)
    }
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = Date, js_name = now)]
    fn date_now() -> f64;
}
#[cfg(target_arch = "wasm32")]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Instant(u64);
#[cfg(target_arch = "wasm32")]
impl Instant {
    pub fn now() -> Self {
        Self(date_now() as u64)
    }
    pub fn duration_since(&self, earlier: Instant) -> Duration {
        Duration::from_millis(self.0 - earlier.0)
    }
    pub fn elapsed(&self) -> Duration {
        Self::now().duration_since(*self)
    }
    pub fn checked_add(&self, duration: Duration) -> Option<Self> {
        match duration.as_millis().try_into() {
            Ok(duration) => self.0.checked_add(duration).map(|i| Self(i)),
            Err(_) => None,
        }
    }
    pub fn checked_sub(&self, duration: Duration) -> Option<Self> {
        match duration.as_millis().try_into() {
            Ok(duration) => self.0.checked_sub(duration).map(|i| Self(i)),
            Err(_) => None,
        }
    }
}

impl Add<Duration> for Instant {
    type Output = Instant;
    fn add(self, other: Duration) -> Instant {
        self.checked_add(other).unwrap()
    }
}
impl Sub<Duration> for Instant {
    type Output = Instant;
    fn sub(self, other: Duration) -> Instant {
        self.checked_sub(other).unwrap()
    }
}
impl Sub<Instant> for Instant {
    type Output = Duration;
    fn sub(self, other: Instant) -> Duration {
        self.duration_since(other)
    }
}
impl AddAssign<Duration> for Instant {
    fn add_assign(&mut self, other: Duration) {
        *self = *self + other;
    }
}
impl SubAssign<Duration> for Instant {
    fn sub_assign(&mut self, other: Duration) {
        *self = *self - other;
    }
}
