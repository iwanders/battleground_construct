use super::primitives::*;
use cgmath::Deg;
use engine::prelude::*;

#[derive(Copy, Debug, Clone)]
pub struct ArtilleryTurret {}
impl Default for ArtilleryTurret {
    fn default() -> Self {
        ArtilleryTurret::new()
    }
}

impl ArtilleryTurret {
    pub fn new() -> Self {
        ArtilleryTurret {}
    }
}
impl Component for ArtilleryTurret {}

impl Drawable for ArtilleryTurret {
    fn drawables(&self) -> Vec<Element> {
        let material: Material = Color {
            r: 200,
            g: 100,
            b: 0,
            a: 255,
        }
        .into();
        let standing_height = 0.75;
        let standing_part = |y: f32| Element {
            transform: Mat4::from_translation(Vec3::new(0.0, y, standing_height / 2.0)),
            primitive: Primitive::Cuboid(Cuboid {
                length: 0.3,
                width: 0.05,
                height: standing_height,
            }),
            material,
        };
        vec![
            // Cylinder at the base
            Element {
                transform: Mat4::from_angle_y(Deg(-90.0))
                    * Mat4::from_translation(Vec3::new(0.0, 0.0, 0.0)),
                primitive: Primitive::Cylinder(Cylinder {
                    radius: 0.4,
                    height: 0.1,
                }),
                material,
            },
            // a totally disjoint circle capping the cylinder!
            Element {
                transform: Mat4::from_translation(Vec3::new(0.0, 0.0, 0.1)),
                primitive: Primitive::Circle(Circle { radius: 0.4 }),
                material,
            },
            // Risers
            standing_part(0.34),
            standing_part(-0.34),
            // Joints with the barrel
            Element {
                transform: Mat4::from_translation(Vec3::new(0.0, 0.34 - 0.05 / 2.0, 0.6)),
                primitive: Primitive::Cuboid(Cuboid {
                    length: 0.2,
                    width: 0.02,
                    height: 0.2,
                }),
                material,
            },
            Element {
                transform: Mat4::from_translation(Vec3::new(0.0, -0.34 + 0.05 / 2.0, 0.6)),
                primitive: Primitive::Cuboid(Cuboid {
                    length: 0.2,
                    width: 0.02,
                    height: 0.2,
                }),
                material,
            },
        ]
    }
}
