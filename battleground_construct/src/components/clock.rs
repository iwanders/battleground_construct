use engine::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Copy, Debug, Clone)]
pub struct Clock {
    current: f32,
    step: f32,
}
impl Default for Clock {
    fn default() -> Self {
        Clock::new()
    }
}

impl Clock {
    pub fn new() -> Self {
        Clock {
            current: 0.0,
            step: 0.001,
        }
    }
    pub fn step_as_f32(&self) -> f32 {
        self.step
    }
    pub fn elapsed_as_f32(&self) -> f32 {
        self.current
    }
    pub fn tick(&mut self) {
        self.current += self.step;
    }
}

impl Component for Clock {}

use crate::components::unit_interface::{Register, RegisterMap, UnitModule};

#[derive(Default)]
pub struct ClockModule {}

impl ClockModule {
    pub fn new() -> Self {
        ClockModule {}
    }
}

impl UnitModule for ClockModule {
    fn get_registers(&self, world: &World, registers: &mut RegisterMap) {
        if let Some((_entity, clock)) = world.component_iter::<Clock>().next() {
            registers.insert(
                battleground_unit_control::modules::clock::REG_CLOCK_ELAPSED,
                Register::new_f32("elapsed", clock.elapsed_as_f32()),
            );
        }
    }
}
