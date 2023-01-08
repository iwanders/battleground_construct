use engine::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MatchTimeLimit {
    time_limit: f32,
    current_time: f32,
}

impl MatchTimeLimit {
    pub fn new(time_limit: f32) -> Self {
        Self {
            time_limit,
            current_time: 0.0,
        }
    }

    pub fn set_time(&mut self, current_time: f32) {
        self.current_time = current_time;
    }

    pub fn is_finished(&self) -> bool {
        self.current_time >= self.time_limit
    }
}
impl Component for MatchTimeLimit {}
