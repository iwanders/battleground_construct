use engine::prelude::*;

#[derive(Copy, Debug, Clone)]
pub struct HitBox {
    length: f32,
    width: f32,
    height: f32,
}

impl HitBox {
    pub fn new(length: f32, width: f32, height: f32) -> Self {
        HitBox { length, width, height }
    }

    pub fn length(&self) -> f32 {
        self.length
    }
    pub fn width(&self) -> f32 {
        self.width
    }
    pub fn height(&self) -> f32 {
        self.height
    }
}
impl Component for HitBox {}
