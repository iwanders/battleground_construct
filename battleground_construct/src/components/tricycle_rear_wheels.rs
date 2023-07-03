use engine::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct TricycleRearWheels {
    wheels: Vec<EntityId>,
    track_width: f32,
}

impl TricycleRearWheels {
    pub fn new(wheels: &[EntityId], track_width: f32) -> Self {
        TricycleRearWheels {
            wheels: wheels.to_vec(),
            track_width,
        }
    }
    pub fn wheels(&self) -> &[EntityId] {
        &self.wheels
    }
    pub fn track_width(&self) -> f32 {
        self.track_width
    }
}
impl Component for TricycleRearWheels {}
