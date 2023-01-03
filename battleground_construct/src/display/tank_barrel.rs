use super::primitives::*;
use crate::components::hit_box::HitBox;
use engine::prelude::*;

#[derive(Copy, Debug, Clone)]
pub struct TankBarrel {
    pub width: f32,
    pub length: f32,
    pub height: f32,
}
impl Default for TankBarrel {
    fn default() -> Self {
        TankBarrel::new()
    }
}

impl TankBarrel {
    pub fn new() -> Self {
        TankBarrel {
            width: 0.1,
            length: 1.0,
            height: 0.1,
        }
    }
    pub fn hitbox(&self) -> HitBox {
        HitBox::new(self.length, self.width, self.height)
    }
}
impl Component for TankBarrel {}

impl Drawable for TankBarrel {
    fn drawables(&self) -> Vec<Element> {
        vec![Element {
            transform: Mat4::from_translation(Vec3::new(self.length / 2.0, 0.0, 0.0)),
            primitive: Primitive::Cuboid(Cuboid {
                width: self.width,
                height: self.height,
                length: self.length,
            }),
            material: Color {
                r: 200,
                g: 100,
                b: 0,
                a: 255,
            }
            .into(),
        }]
    }
}
