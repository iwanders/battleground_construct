use super::primitives::*;
use engine::prelude::*;

#[derive(Copy, Debug, Clone)]
pub struct DebugBox {
    pub width: f32,
    pub length: f32,
    pub height: f32,
}

impl DebugBox {
    pub fn from_size(edge: f32) -> Self {
        DebugBox {
            width: edge,
            length: edge,
            height: edge,
        }
    }
    pub fn new() -> Self {
        DebugBox {
            width: 1.0,
            length: 1.0,
            height: 1.0,
        }
    }
}
impl Component for DebugBox {}

impl Drawable for DebugBox {
    fn drawables(&self) -> Vec<Element> {
        vec![Element {
            transform: Mat4::from_translation(Vec3::new(0.0, 0.0, 0.0)),
            primitive: Primitive::Cuboid(Cuboid {
                width: self.width,
                height: self.height,
                length: self.length,
            }),
            color: Color {
                r: 255,
                g: 0,
                b: 255,
                a: 128,
            },
        }]
    }
}
