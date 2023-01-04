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
    pub fn hit_boxes(&self) -> Vec<(Mat4, HitBox)> {
        vec![(
            Mat4::from_translation(Vec3::new(self.length / 2.0, 0.0, 0.0)),
            HitBox::new(self.length, self.width, self.height),
        )]
    }
}
impl Component for TankBarrel {}

impl Drawable for TankBarrel {
    fn drawables(&self) -> Vec<Element> {
        self.hit_boxes()
            .iter()
            .map(|(t, b)| Element {
                transform: *t,
                primitive: Primitive::Cuboid(Cuboid {
                    width: b.width(),
                    height: b.height(),
                    length: b.length(),
                }),
                material: Color {
                    r: 200,
                    g: 100,
                    b: 0,
                    a: 255,
                }
                .into(),
            })
            .collect()
    }
}
