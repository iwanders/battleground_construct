use engine::prelude::*;

#[derive(Copy, Debug, Clone)]
pub struct Cannon {
    reload_time: f32,
    last_fire_time: f32,
    muzzle_velocity: f32,
}

impl Cannon {
    pub fn new() -> Self {
        Self {
            reload_time: 1.0,
            last_fire_time: -2.0, // spawn ready to fire.
            muzzle_velocity: 10.0,
        }
    }
    pub fn is_ready(&self, current_time: f32) -> bool {
        (current_time - self.last_fire_time) > self.reload_time
    }
    pub fn fire(&mut self, current_time: f32) {
        self.last_fire_time = current_time;
    }
    pub fn muzzle_velocity(&self) -> f32 {
        self.muzzle_velocity
    }
}
impl Component for Cannon {}
