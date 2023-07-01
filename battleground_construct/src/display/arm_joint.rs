use super::primitives::*;
use cgmath::Deg;
use engine::prelude::*;

#[derive(Copy, Debug, Clone)]
pub struct ArmJoint {
    inline: bool,
}
impl Default for ArmJoint {
    fn default() -> Self {
        ArmJoint::new()
    }
}

impl ArmJoint {
    pub fn new() -> Self {
        ArmJoint { inline: false }
    }
    pub fn inline(mut self) -> Self {
        self.inline = true;
        self
    }
}
impl Component for ArmJoint {}

impl Drawable for ArmJoint {
    fn drawables(&self) -> Vec<Element> {
        let material: Material = Color {
            r: 200,
            g: 200,
            b: 200,
            a: 255,
        }
        .into();

        let emissive_color = Color {
            r: 200,
            g: 200,
            b: 200,
            a: 255,
        };

        let emissive_material = Material::FlatMaterial(FlatMaterial {
            color: emissive_color,
            is_emissive: true,
            emissive: emissive_color,
            ..Default::default()
        });

        let radius = 0.1;
        let length = 0.2;
        let x = -length / 2.0;

        let t = if self.inline {
            Mat4::from_angle_y(Deg(0.0))
        } else {
            Mat4::from_angle_z(Deg(-90.0))
        };

        vec![
            // Cylinder at the base
            Element {
                transform: t * Mat4::from_translation(Vec3::new(x, 0.0, 0.0)),
                primitive: Primitive::Cylinder(Cylinder {
                    radius,
                    height: length,
                }),
                material,
            },
            // a totally disjoint circle capping the cylinder!
            Element {
                transform: t
                    * Mat4::from_angle_y(Deg(-90.0))
                    * Mat4::from_translation(Vec3::new(0.0, 0.0, x)),
                primitive: Primitive::Circle(Circle { radius }),
                material,
            },
            Element {
                transform: t
                    * Mat4::from_angle_y(Deg(-90.0))
                    * Mat4::from_translation(Vec3::new(0.0, 0.0, -x)),
                primitive: Primitive::Circle(Circle { radius }),
                material,
            },
            Element {
                transform: t * Mat4::from_translation(Vec3::new(x + length / 2.0, 0.0, 0.0)),
                primitive: Primitive::Cuboid(Cuboid {
                    length: length * 0.75,
                    width: radius * 2.0 + 0.05,
                    height: 0.02,
                }),
                material: emissive_material,
            },
        ]
    }
}
