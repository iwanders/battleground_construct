use engine::prelude::*;

#[derive(Copy, Debug, Clone)]
pub struct CapturePoint {
    radius: f32,
    speed: f32,
}

impl CapturePoint {
    pub fn new(radius: f32, speed: f32) -> Self {
        CapturePoint { radius, speed }
    }
    pub fn radius(&self) -> f32 {
        self.radius
    }
    pub fn speed(&self) -> f32 {
        self.speed
    }
}

impl Component for CapturePoint {}
