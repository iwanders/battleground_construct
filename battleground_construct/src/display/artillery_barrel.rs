use super::primitives::*;
use crate::components::hit_box::HitBox;
use engine::prelude::*;

#[derive(Copy, Debug, Clone)]
pub struct ArtilleryBarrel {
    battery_width: f32,
    battery_height: f32,
    battery_length: f32,
}
impl Default for ArtilleryBarrel {
    fn default() -> Self {
        ArtilleryBarrel::new()
    }
}

pub const BARREL_WIDTH: f32 = 0.6;
pub const BARREL_HORIZONTAL_SLOTS: u32 = 4;
pub const BARREL_HORIZONTAL_OFFSET: f32 = BARREL_WIDTH / BARREL_HORIZONTAL_SLOTS as f32;
pub const BARREL_HEIGHT: f32 = 0.6;
pub const BARREL_VERTICAL_SLOTS: u32 = 4;
pub const BARREL_VERTICAL_OFFSET: f32 = BARREL_HEIGHT / BARREL_VERTICAL_SLOTS as f32;

impl ArtilleryBarrel {
    pub fn new() -> Self {
        ArtilleryBarrel {
            battery_width: BARREL_WIDTH,
            battery_height: BARREL_HEIGHT,
            battery_length: 0.8,
        }
    }

    pub fn hitbox(&self) -> HitBox {
        HitBox::new(self.battery_length, self.battery_width, self.battery_height)
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
        let battery_width = self.battery_width;
        let battery_height = self.battery_height;
        let battery_length = self.battery_length;

        let panel_thickness = 0.01;
        let mut res = vec![];
        let ylower = (BARREL_HORIZONTAL_SLOTS / 2) as i32;
        for y in -ylower..=ylower {
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
        let zlower = (BARREL_VERTICAL_SLOTS / 2) as i32;
        for z in -zlower..=zlower {
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
        // Cool... but now we have empty tubes... insert two panes to make them appear black.
        res.push(Element {
            transform: Mat4::from_translation(Vec3::new((battery_length - 0.2) / 2.0, 0.0, 0.0)),
            primitive: Primitive::Cuboid(Cuboid {
                length: panel_thickness,
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
        res.push(Element {
            transform: Mat4::from_translation(Vec3::new(-(battery_length - 0.2) / 2.0, 0.0, 0.0)),
            primitive: Primitive::Cuboid(Cuboid {
                length: panel_thickness,
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
