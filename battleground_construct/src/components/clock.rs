use engine::prelude::*;

pub struct Clock {
    epoch: std::time::Instant,
    current: std::time::Instant,
    step: std::time::Duration,
}

impl Clock {
    pub fn new() -> Self {
        let v = std::time::Instant::now();
        Clock {
            epoch: v,
            current: v,
            step: std::time::Duration::from_secs_f32(0.01),
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
}

impl Component for Clock {}

pub struct ClockSystem {}
impl System for ClockSystem {
    fn update(&mut self, world: &mut World) {
        let (_entity, mut clock) = world
            .component_iter_mut::<Clock>()
            .next()
            .expect("Should have one clock");
        clock.tick();
    }
}
