use engine::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Copy, Debug, Clone)]
pub struct CapturePoint {
    radius: f32,
    capture_speed: f32,
}

impl CapturePoint {
    pub fn new(radius: f32, capture_speed: f32) -> Self {
        CapturePoint {
            radius,
            capture_speed,
        }
    }
    pub fn radius(&self) -> f32 {
        self.radius
    }
    pub fn capture_speed(&self) -> f32 {
        self.capture_speed
    }
}

impl Component for CapturePoint {}
