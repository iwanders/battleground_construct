use super::primitives::*;
use engine::prelude::*;

#[derive(Copy, Debug, Clone)]
pub struct ArtilleryBarrel {}
impl Default for ArtilleryBarrel {
    fn default() -> Self {
        ArtilleryBarrel::new()
    }
}

impl ArtilleryBarrel {
    pub fn new() -> Self {
        ArtilleryBarrel {}
    }
}
impl Component for ArtilleryBarrel {}

impl Drawable for ArtilleryBarrel {
    fn drawables(&self) -> Vec<Element> {
        let material: Material = Color {
            r: 40,
            g: 40,
            b: 40,
            a: 255,
        }
        .into();
        let battery_width = 0.6;
        let battery_height = 0.6;
        let battery_length = 0.8;

        let panel_thickness = 0.01;
        let mut res = vec![];
        for y in -2..=2 {
            let y_pos = (y as f32) * battery_width / 4.0;
            res.push(Element {
                transform: Mat4::from_translation(Vec3::new(0.0, y_pos, 0.0)),
                primitive: Primitive::Cuboid(Cuboid {
                    length: battery_length,
                    width: panel_thickness,
                    height: battery_height
                        - if y == -2 || y == 2 {
                            panel_thickness
                        } else {
                            -panel_thickness
                        },
                }),
                material,
            })
        }
        for z in -2..=2 {
            let z_pos = (z as f32) * battery_width / 4.0;
            res.push(Element {
                transform: Mat4::from_translation(Vec3::new(0.0, 0.0, z_pos)),
                primitive: Primitive::Cuboid(Cuboid {
                    length: battery_length,
                    height: panel_thickness,
                    width: battery_width
                        + if z == -2 || z == 2 {
                            panel_thickness
                        } else {
                            -panel_thickness
                        },
                }),
                material,
            })
        }
        // Cool... but now we have empty tubes... insert this super ugly cuboid to fill them for now.
        res.push(Element {
            transform: Mat4::from_translation(Vec3::new(0.0, 0.0, 0.0)),
            primitive: Primitive::Cuboid(Cuboid {
                length: battery_length - 0.2,
                height: battery_height - panel_thickness,
                width: battery_width - panel_thickness,
            }),
            material: Color {
                r: 10,
                g: 10,
                b: 10,
                a: 255,
            }
            .into(),
        });
        res
    }
}
