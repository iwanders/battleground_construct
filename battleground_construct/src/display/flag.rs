use super::primitives::*;
use engine::prelude::*;

#[derive(Copy, Debug, Clone)]
pub struct Flag {
    pub pole_height: f32,
    // 0.0-1.0
    pub flag_position: f32,
    pub radius: f32,
    pub flag_color: Color,
    pub pole_color: Color,
}

impl Default for Flag {
    fn default() -> Self {
        Flag {
            pole_height: 1.0,
            flag_position: 1.0,
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
        res.pole_height *= scale;
        res.radius *= scale;
        res.flag_color = color;
        res
    }

    pub fn set_pole_height(&mut self, pole_height: f32) {
        self.pole_height = pole_height;
    }

    pub fn set_color(&mut self, color: Color) {
        self.flag_color = color;
    }

    pub fn set_flag_position(&mut self, flag_position: f32) {
        self.flag_position = flag_position;
    }
}
impl Component for Flag {}

impl Drawable for Flag {
    fn drawables(&self) -> Vec<Element> {
        let flag_width = self.pole_height * 0.4;
        let flag_height = self.pole_height * 0.3;

        let flag_min_z = flag_height / 2.0;
        let flag_max_z = self.pole_height - self.radius * 2.0 - flag_height / 2.0;
        let flag_z = (flag_max_z - flag_min_z) * self.flag_position + flag_min_z;

        let addition = Color::rgb(10, 10, 10);

        vec![
            Element {
                transform: Mat4::from_translation(Vec3::new(0.0, 0.0, self.pole_height))
                    * Mat4::from_angle_y(cgmath::Deg(90.0)),
                primitive: Primitive::Cylinder(Cylinder {
                    radius: self.radius,
                    height: self.pole_height,
                }),
                material: self.pole_color.into(),
            },
            Element {
                transform: Mat4::from_translation(Vec3::new(0.0, 0.0, self.pole_height)),
                primitive: Primitive::Sphere(Sphere {
                    radius: self.radius * 2.0,
                }),
                material: self.pole_color.into(),
            },
            Element {
                transform: Mat4::from_translation(Vec3::new(0.0 + flag_width / 2.0, 0.0, flag_z)),
                primitive: Primitive::Cuboid(Cuboid {
                    width: self.radius * 0.5,
                    length: flag_width,
                    height: flag_height,
                }),
                material: self.flag_color.saturating_add(&addition).into(),
            },
        ]
    }
}
