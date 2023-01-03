use super::primitives::*;
use crate::components::hit_box::HitBox;
use engine::prelude::*;

#[derive(Copy, Debug, Clone)]
pub struct TankTurret {
    pub width: f32,
    pub length: f32,
    pub height: f32,
}
impl Default for TankTurret {
    fn default() -> Self {
        TankTurret::new()
    }
}

impl TankTurret {
    pub fn new() -> Self {
        TankTurret {
            width: 0.5,
            length: 0.7,
            height: 0.1,
        }
    }
    pub fn hitbox(&self) -> HitBox {
        HitBox::new(self.length, self.width, self.height)
    }
}
impl Component for TankTurret {}

impl Drawable for TankTurret {
    fn drawables(&self) -> Vec<Element> {
        vec![Element {
            transform: Mat4::from_translation(Vec3::new(0.0, 0.0, 0.0)),
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
