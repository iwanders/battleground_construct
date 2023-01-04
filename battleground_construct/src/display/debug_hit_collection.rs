use super::primitives::*;
use crate::components::hit_box::HitBox;
use engine::prelude::*;
#[derive(Debug, Clone, Default)]
pub struct DebugHitCollection {
    hit_boxes: Vec<(Mat4, Cuboid)>,
}

impl DebugHitCollection {
    pub fn from_hit_boxes(hit_boxes: &[(Mat4, HitBox)]) -> Self {
        DebugHitCollection {
            hit_boxes: hit_boxes
                .iter()
                .map(|(t, b)| {
                    (
                        *t,
                        Cuboid {
                            length: b.length() + 0.01,
                            width: b.width() + 0.01,
                            height: b.height() + 0.01,
                        },
                    )
                })
                .collect(),
        }
    }
}
impl Component for DebugHitCollection {}

impl Drawable for DebugHitCollection {
    fn drawables(&self) -> Vec<Element> {
        let material: Material = Color {
            r: 255,
            g: 0,
            b: 255,
            a: 128,
        }
        .into();

        self.hit_boxes
            .iter()
            .map(|(transform, c)| Element {
                transform: *transform,
                primitive: Primitive::Cuboid(*c),
                material,
            })
            .collect()
    }
}
