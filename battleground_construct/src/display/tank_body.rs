use super::primitives::*;
use engine::prelude::*;

#[derive(Copy, Debug, Clone)]
pub struct TankBody {
    pub width: f32,
    pub length: f32,
    pub height: f32,
    pub z_offset: f32,
    color: Color,
}

impl TankBody {
    pub fn new() -> Self {
        TankBody {
            width: 1.0,
            length: 2.0,
            height: 0.25,
            z_offset: 0.25,
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
}
impl Component for TankBody {}

impl Drawable for TankBody {
    fn drawables(&self) -> Vec<Element> {
        vec![Element {
            transform: Mat4::from_translation(Vec3::new(0.0, 0.0, self.z_offset)),
            primitive: Primitive::Cuboid(Cuboid {
                width: self.width,
                height: self.height,
                length: self.length,
            }),
            color: self.color,
        }]
    }
}
