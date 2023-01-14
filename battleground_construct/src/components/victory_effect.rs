use engine::prelude::*;

#[derive(Debug, Clone)]
pub struct VictoryEffect {
    last_effect_time: f32,
    effect_interval: f32,
}
impl Default for VictoryEffect {
    fn default() -> Self {
        Self {
            last_effect_time: 0.0,
            effect_interval: 3.0,
        }
    }
}

impl VictoryEffect {
    pub fn update(&mut self, time: f32) -> bool {
        if (self.last_effect_time + self.effect_interval) < time {
            self.last_effect_time = time;
            return true;
        }
        false
    }
}
impl Component for VictoryEffect {}
