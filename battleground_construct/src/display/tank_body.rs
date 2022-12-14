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
impl Default for TankBody {
    fn default() -> Self {
        TankBody::new()
    }
}

impl TankBody {
    pub fn new() -> Self {
        TankBody {
            length: 2.0,
            width: 1.0,
            height: 0.25,
            color: Color {
                r: 0,
                g: 20,
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
        let addition = Color::rgb(10, 10, 10);
        let emissive_material = Material::FlatMaterial(FlatMaterial {
            color: self.color, // .saturating_add(&addition)
            is_emissive: true,
            emissive: self.color.saturating_add(&addition),
            ..Default::default()
        });

        vec![
            Element {
                transform: Mat4::from_translation(Vec3::new(0.0, 0.0, 0.0)),
                primitive: Primitive::Cuboid(Cuboid {
                    length: self.length,
                    width: self.width,
                    height: self.height,
                }),
                material: self.color.into(),
            },
            Element {
                transform: Mat4::from_translation(Vec3::new(0.0, 0.4, 0.125 + 0.01)),
                primitive: Primitive::Cuboid(Cuboid {
                    length: self.length - 0.25,
                    width: 0.1,
                    height: 0.02,
                }),
                material: emissive_material,
            },
            Element {
                transform: Mat4::from_translation(Vec3::new(0.0, -0.4, 0.125 + 0.01)),
                primitive: Primitive::Cuboid(Cuboid {
                    length: self.length - 0.25,
                    width: 0.1,
                    height: 0.02,
                }),
                material: emissive_material,
            },
        ]
    }
}
