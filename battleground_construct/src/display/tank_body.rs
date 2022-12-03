use super::primitives::*;
use crate::components::hit_box::HitBox;
use engine::prelude::*;

#[derive(Copy, Debug, Clone)]
pub struct TankBody {
    pub length: f32,
    pub width: f32,
    pub height: f32,
    color: Color,
}

impl TankBody {
    pub fn new() -> Self {
        TankBody {
            length: 2.0,
            width: 1.0,
            height: 0.25,
            color: Color {
                r: 0,
                g: 255,
                b: 0,
                a: 255,
            },
        }
    }
    pub fn set_color(&mut self, color: Color) {
        self.color = color;
    }

    pub fn hitbox(&self) -> HitBox {
        HitBox::new(self.length, self.width, self.height)
    }
}
impl Component for TankBody {}

impl Drawable for TankBody {
    fn drawables(&self) -> Vec<Element> {
        vec![Element {
            transform: Mat4::from_translation(Vec3::new(0.0, 0.0, 0.0)),
            primitive: Primitive::Cuboid(Cuboid {
                length: self.length,
                width: self.width,
                height: self.height,
            }),
            color: self.color,
        }]
    }
}
