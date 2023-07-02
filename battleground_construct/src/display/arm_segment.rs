use super::primitives::*;
use crate::components::hit_box::HitBox;
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
    pub fn hit_boxes(&self) -> Vec<(Mat4, HitBox)> {
        vec![(
            Mat4::from_translation(Vec3::new(1.0 / 2.0, 0.0, 0.0)),
            HitBox::new(1.0, 0.05 * 2.0, 0.05 * 2.0),
        )]
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
            // Cap at the end.
            Element {
                transform: Mat4::from_translation(Vec3::new(length, 0.0, 0.0))
                    * Mat4::from_angle_y(cgmath::Deg(90.0)),
                primitive: Primitive::Circle(Circle { radius }),
                material,
            },
        ]
    }
}
