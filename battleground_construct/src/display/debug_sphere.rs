use super::primitives::*;
use engine::prelude::*;

#[derive(Copy, Debug, Clone)]
pub struct DebugSphere {
    pub radius: f32,
}
impl Default for DebugSphere {
    fn default() -> Self {
        DebugSphere { radius: 1.0 }
    }
}

impl DebugSphere {
    pub fn with_radius(radius: f32) -> Self {
        DebugSphere { radius }
    }
    pub fn new() -> Self {
        Default::default()
    }
}
impl Component for DebugSphere {}

impl Drawable for DebugSphere {
    fn drawables(&self) -> Vec<Element> {
        vec![Element {
            transform: Mat4::from_translation(Vec3::new(0.0, 0.0, 0.0)),
            primitive: Primitive::Sphere(Sphere {
                radius: self.radius,
            }),
            material: Color {
                r: 255,
                g: 0,
                b: 255,
                a: 128,
            }
            .into(),
        }]
    }
}
