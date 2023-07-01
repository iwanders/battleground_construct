use super::primitives::*;
use engine::prelude::*;

#[derive(Copy, Debug, Clone)]
pub struct ArmSegment {}
impl Default for ArmSegment {
    fn default() -> Self {
        ArmSegment::new()
    }
}

impl ArmSegment {
    pub fn new() -> Self {
        ArmSegment {}
    }
}
impl Component for ArmSegment {}

impl Drawable for ArmSegment {
    fn drawables(&self) -> Vec<Element> {
        let material: Material = Color {
            r: 200,
            g: 200,
            b: 200,
            a: 255,
        }
        .into();

        let radius = 0.05;
        let length = 1.0;

        vec![
            // Cylinder at the base
            Element {
                transform: Mat4::from_translation(Vec3::new(0.0, 0.0, 0.0)),
                primitive: Primitive::Cylinder(Cylinder {
                    radius,
                    height: length,
                }),
                material,
            },
        ]
    }
}
