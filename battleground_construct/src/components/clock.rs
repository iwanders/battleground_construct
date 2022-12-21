use engine::prelude::*;

#[derive(Copy, Debug, Clone)]
pub struct Clock {
    epoch: std::time::Instant,
    current: std::time::Instant,
    step: std::time::Duration,
}
impl Default for Clock {
    fn default() -> Self {
        Clock::new()
    }
}

impl Clock {
    pub fn new() -> Self {
        let v = std::time::Instant::now();
        Clock {
            epoch: v,
            current: v,
            step: std::time::Duration::from_secs_f32(0.001),
        }
    }
    pub fn step_as_f32(&self) -> f32 {
        self.step.as_secs_f32()
    }
    pub fn elapsed_as_f32(&self) -> f32 {
        (self.current - self.epoch).as_secs_f32()
    }
    pub fn tick(&mut self) {
        self.current += self.step;
    }

    pub fn ratio_of_realtime(&self) -> f32 {
        let sim = (self.current - self.epoch).as_secs_f32();
        let realtime = (std::time::Instant::now() - self.epoch).as_secs_f32();
        sim / realtime
    }
}

impl Component for Clock {}

use crate::components::vehicle_interface::{Register, RegisterMap, VehicleModule};

#[derive(Default)]
pub struct ClockModule {}

impl ClockModule {
    pub fn new() -> Self {
        ClockModule {}
    }
}

impl VehicleModule for ClockModule {
    fn get_registers(&self, world: &World, registers: &mut RegisterMap) {
        if let Some((_entity, clock)) = world.component_iter_mut::<Clock>().next() {
            registers.insert(0, Register::new_f32("elapsed", clock.elapsed_as_f32()));
        }
    }
}
