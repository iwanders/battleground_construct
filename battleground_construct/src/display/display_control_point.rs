use super::primitives::*;
use engine::prelude::*;

#[derive(Copy, Debug, Clone)]
pub struct DisplayControlPoint {
    pub radius: f32,
    color: Color,
}
impl Default for DisplayControlPoint {
    fn default() -> Self {
        DisplayControlPoint::new()
    }
}

impl DisplayControlPoint {
    pub fn new() -> Self {
        DisplayControlPoint {
            radius: 1.0,
            color: Color {
                r: 128,
                g: 128,
                b: 128,
                a: 255,
            },
        }
    }
    pub fn set_color(&mut self, color: Color) {
        // self.color = color;
        // self.color.a = 128;
        // self.color.r = ((color.r as u32 + 128) / 2) as u8;
        // self.color.g = ((color.g as u32 + 128) / 2) as u8;
        // self.color.b = ((color.b as u32 + 128) / 2) as u8;
        // self.color.a = ((color.a as u32 + 128) / 2) as u8;
        self.color.r = color.r as u8;
        self.color.g = color.g as u8;
        self.color.b = color.b as u8;
    }

    pub fn set_radius(&mut self, radius: f32) {
        self.radius = radius;
    }
}
impl Component for DisplayControlPoint {}

impl Drawable for DisplayControlPoint {
    fn drawables(&self) -> Vec<Element> {
        let material = Material::FenceMaterial(FenceMaterial {
            color: self.color,
            ..Default::default()
        });

        vec![Element {
            transform: Mat4::from_angle_y(cgmath::Deg(-90.0)),
            primitive: Primitive::Cylinder(Cylinder {
                radius: self.radius,
                height: 0.5,
            }),
            material,
        }]
    }
}
