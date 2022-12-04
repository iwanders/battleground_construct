use super::primitives::*;
use crate::components::hit_box::HitBox;
use engine::prelude::*;

#[derive(Copy, Debug, Clone)]
pub struct Radar {
    pub length: f32,
    pub width: f32,
    pub height: f32,
    pub angle: f32,
    color: Color,
}

impl Radar {
    pub fn new() -> Self {
        Radar {
            length: 0.15,
            width: 0.01,
            height: 0.05,
            angle: 80.0f32.to_radians(),
            color: Color {
                r: 0,
                g: 0,
                b: 255,
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
impl Component for Radar {}

impl Drawable for Radar {
    fn drawables(&self) -> Vec<Element> {
        let ls = self.length / 2.0 * self.angle.sin();
        let lc = self.length / 2.0 * self.angle.cos();
        vec![
            Element {
                transform: Mat4::from_translation(Vec3::new(lc, ls, 0.0))
                    * Mat4::from_angle_z(cgmath::Rad(self.angle)),
                primitive: Primitive::Cuboid(Cuboid {
                    length: self.length,
                    width: self.width,
                    height: self.height,
                }),
                color: self.color,
            },
            Element {
                transform: Mat4::from_translation(Vec3::new(lc, -ls, 0.0))
                    * Mat4::from_angle_z(-cgmath::Rad(self.angle)),
                primitive: Primitive::Cuboid(Cuboid {
                    length: self.length,
                    width: self.width,
                    height: self.height,
                }),
                color: self.color,
            },
        ]
    }
}
