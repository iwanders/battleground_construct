use super::primitives::*;
use engine::prelude::*;

#[derive(Copy, Debug, Clone)]
pub struct DebugBox {
    pub length: f32,
    pub width: f32,
    pub height: f32,
}

impl DebugBox {
    pub fn cube(edge: f32) -> Self {
        DebugBox {
            length: edge,
            width: edge,
            height: edge,
        }
    }
    pub fn new(length: f32, width: f32, height: f32) -> Self {
        DebugBox {
            length,
            width,
            height,
        }
    }
    pub fn grown(self, value: f32) -> Self{
        DebugBox {
            length: self.length+value,
            width: self.width+value,
            height: self.height+value,
        }
    }
}
impl Component for DebugBox {}

impl Drawable for DebugBox {
    fn drawables(&self) -> Vec<Element> {
        vec![Element {
            transform: Mat4::from_translation(Vec3::new(0.0, 0.0, 0.0)),
            primitive: Primitive::Cuboid(Cuboid {
                length: self.length,
                width: self.width,
                height: self.height,
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
