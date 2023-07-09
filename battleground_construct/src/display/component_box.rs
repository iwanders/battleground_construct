use super::primitives::*;
use crate::components::hit_box::HitBox;
use crate::components::hit_collection::HitCollection;
use engine::prelude::*;

/*
Exterior dimensions of the box.
                        ^ height
                        |
                        |
     <- height / 2.    base     -> height / 2.0
*/

const WALL_THICKNESS: f32 = 0.02;

#[derive(Copy, Debug, Clone)]
pub struct ComponentBox {
    pub width: f32,
    pub length: f32,
    pub height: f32,
}

impl ComponentBox {
    pub fn from_config(config: crate::units::common::ComponentBoxSpawnConfig) -> Self {
        ComponentBox {
            width: config.width,
            length: config.length,
            height: config.height,
        }
    }
    pub fn hit_collection(&self) -> HitCollection {
        HitCollection::from_hit_boxes(&[(
            Mat4::from_translation(Vec3::new(0.0, 0.0, self.height / 2.0)),
            HitBox::new(self.length, self.width, self.height),
        )])
    }
}
impl Component for ComponentBox {}

impl Drawable for ComponentBox {
    fn drawables(&self) -> Vec<Element> {
        let wall_offset = self.height / 2.0;
        let y = self.width / 2.0;
        let x = self.length / 2.0 - WALL_THICKNESS;
        let material: Material = Color {
            r: 200,
            g: 100,
            b: 0,
            a: 255,
        }
        .into();

        vec![
            Element {
                transform: Mat4::from_translation(Vec3::new(
                    0.0,
                    -y + WALL_THICKNESS / 2.0,
                    wall_offset,
                )),
                primitive: Primitive::Cuboid(Cuboid {
                    width: WALL_THICKNESS,
                    height: self.height,
                    length: self.length - WALL_THICKNESS * 2.0,
                }),
                material,
            },
            Element {
                transform: Mat4::from_translation(Vec3::new(
                    0.0,
                    y - WALL_THICKNESS / 2.0,
                    wall_offset,
                )),
                primitive: Primitive::Cuboid(Cuboid {
                    width: WALL_THICKNESS,
                    height: self.height,
                    length: self.length - WALL_THICKNESS * 2.0,
                }),
                material,
            },
            Element {
                transform: Mat4::from_translation(Vec3::new(x, 0.0, wall_offset)),
                primitive: Primitive::Cuboid(Cuboid {
                    width: self.width,
                    height: self.height,
                    length: WALL_THICKNESS,
                }),
                material,
            },
            Element {
                transform: Mat4::from_translation(Vec3::new(-x, 0.0, wall_offset)),
                primitive: Primitive::Cuboid(Cuboid {
                    width: self.width,
                    height: self.height,
                    length: WALL_THICKNESS,
                }),
                material,
            },
        ]
    }
}
