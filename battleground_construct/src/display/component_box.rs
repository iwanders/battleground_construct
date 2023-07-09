use super::primitives::*;
use crate::components::hit_box::HitBox;
use crate::components::hit_collection::HitCollection;
use engine::prelude::*;

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
