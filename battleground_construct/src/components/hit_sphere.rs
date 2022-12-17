use engine::prelude::*;

#[derive(Copy, Debug, Clone)]
pub struct HitSphere {
    pub radius: f32,
}

impl Default for HitSphere {
    fn default() -> Self {
        HitSphere::new()
    }
}

impl HitSphere {
    pub fn new() -> Self {
        Self::with_radius(1.0)
    }
    pub fn with_radius(radius: f32) -> Self {
        HitSphere { radius }
    }
    pub fn radius(&self) -> f32 {
        self.radius
    }
}
impl Component for HitSphere {}
