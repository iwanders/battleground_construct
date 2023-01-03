use super::primitives::*;
use crate::components::hit_box::HitBox;
use engine::prelude::*;

#[derive(Copy, Debug, Clone)]
pub struct ArtilleryBody {
    pub length: f32,
    pub width: f32,
    pub height: f32,
    color: Color,
}
impl Default for ArtilleryBody {
    fn default() -> Self {
        ArtilleryBody::new()
    }
}

impl ArtilleryBody {
    pub fn new() -> Self {
        ArtilleryBody {
            length: 2.0,
            width: 1.25,
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
impl Component for ArtilleryBody {}

impl Drawable for ArtilleryBody {
    fn drawables(&self) -> Vec<Element> {
        use cgmath::Deg;
        let addition = Color::rgb(10, 10, 10);
        let emissive_material = Material::FlatMaterial(FlatMaterial {
            color: self.color, // .saturating_add(&addition)
            is_emissive: true,
            emissive: self.color.saturating_add(&addition),
            ..Default::default()
        });
        let base_material: Material = self.color.into();

        let mut elements = vec![
            Element {
                transform: Mat4::from_translation(Vec3::new(0.0, 0.0, 0.0)),
                primitive: Primitive::Cuboid(Cuboid {
                    length: self.length,
                    width: self.width,
                    height: self.height,
                }),
                material: base_material,
            },
            Element {
                transform: Mat4::from_translation(Vec3::new(0.0, 0.4 + 0.125, 0.125 + 0.01)),
                primitive: Primitive::Cuboid(Cuboid {
                    length: self.length - 0.25,
                    width: 0.1,
                    height: 0.02,
                }),
                material: emissive_material,
            },
            Element {
                transform: Mat4::from_translation(Vec3::new(0.0, -0.4 - 0.125, 0.125 + 0.01)),
                primitive: Primitive::Cuboid(Cuboid {
                    length: self.length - 0.25,
                    width: 0.1,
                    height: 0.02,
                }),
                material: emissive_material,
            },
        ];
        for sign in [1.0, -1.0] {
            for offset in [-0.75 + 0.05, 0.75 - 0.05] {
                elements.push(Element {
                    transform: Mat4::from_translation(Vec3::new(
                        offset,
                        sign * self.width / 2.0,
                        -self.height / 2.0 + 0.05,
                    )) * Mat4::from_angle_x(Deg(sign * -30.0)) // rotate down
                        * Mat4::from_angle_z(Deg(sign * offset.signum() * -20.0)) // rotate forwards
                        * Mat4::from_translation(sign * Vec3::new(0.0, 0.09, 0.0)), // translate into track
                    primitive: Primitive::Cuboid(Cuboid {
                        length: 0.5,
                        width: 0.43,
                        height: 0.1,
                    }),
                    material: base_material,
                });
            }
        }

        elements
    }
}
