use super::primitives::*;
use engine::prelude::*;

#[derive(Copy, Debug, Clone)]
pub struct Flag {
    pub height: f32,
    pub radius: f32,
    pub flag_color: Color,
    pub pole_color: Color,
}

impl Default for Flag {
    fn default() -> Self {
        Flag {
            height: 1.0,
            radius: 0.02,
            flag_color: Color {
                r: 255,
                g: 0,
                b: 0,
                a: 255,
            },
            pole_color: Color {
                r: 30,
                g: 30,
                b: 30,
                a: 255,
            },
        }
    }
}

impl Flag {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn from_scale_color(scale: f32, color: Color) -> Self {
        let mut res = Self::default();
        res.height *= scale;
        res.radius *= scale;
        res.flag_color = color;
        res
    }
}
impl Component for Flag {}

impl Drawable for Flag {
    fn drawables(&self) -> Vec<Element> {
        let flag_width = self.height * 0.4;
        let flag_height = self.height * 0.3;
        vec![
            Element {
                transform: Mat4::from_translation(Vec3::new(0.0, 0.0, self.height))
                    * Mat4::from_angle_y(cgmath::Deg(90.0)),
                primitive: Primitive::Cylinder(Cylinder {
                    radius: self.radius,
                    height: self.height,
                }),
                color: self.pole_color,
            },
            Element {
                transform: Mat4::from_translation(Vec3::new(0.0, 0.0, self.height)),
                primitive: Primitive::Sphere(Sphere {
                    radius: self.radius * 2.0,
                }),
                color: self.pole_color,
            },
            Element {
                transform: Mat4::from_translation(Vec3::new(
                    0.0 + flag_width / 2.0,
                    0.0,
                    self.height - self.radius * 2.0 - flag_height / 2.0,
                )),
                primitive: Primitive::Cuboid(Cuboid {
                    width: self.radius * 0.5,
                    length: flag_width,
                    height: flag_height,
                }),
                color: self.flag_color,
            },
        ]
    }
}
