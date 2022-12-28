use engine::prelude::*;

#[derive(Copy, Debug, Clone)]
pub struct SelectBox {
    length: f32,
    width: f32,
    height: f32,
}

impl SelectBox {
    pub fn from_hit_box(hit_box: &super::hit_box::HitBox) -> Self {
        SelectBox {
            length: hit_box.length(),
            width: hit_box.width(),
            height: hit_box.height(),
        }
    }
    pub fn new(length: f32, width: f32, height: f32) -> Self {
        SelectBox {
            length,
            width,
            height,
        }
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
impl Component for SelectBox {}
