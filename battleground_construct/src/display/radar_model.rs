use super::primitives::*;
use engine::prelude::*;

#[derive(Copy, Debug, Clone)]
pub struct RadarModel {
    pub length: f32,
    pub width: f32,
    pub height: f32,
    pub angle: f32,
    color: Color,
}

impl Default for RadarModel {
    fn default() -> Self {
        RadarModel::new()
    }
}

impl RadarModel {
    pub fn new() -> Self {
        RadarModel {
            length: 0.15,
            width: 0.01,
            height: 0.05,
            angle: 80.0f32.to_radians(),
            color: Color {
                r: 0,
                g: 0,
                b: 255,
                a: 255,
            },
        }
    }
    pub fn set_color(&mut self, color: Color) {
        self.color = color;
    }
}
impl Component for RadarModel {}

impl Drawable for RadarModel {
    fn drawables(&self) -> Vec<Element> {
        let ls = self.length / 2.0 * self.angle.sin();
        let lc = self.length / 2.0 * self.angle.cos();
        vec![
            Element {
                transform: Mat4::from_translation(Vec3::new(lc, ls, 0.0))
                    * Mat4::from_angle_z(cgmath::Rad(self.angle)),
                primitive: Primitive::Cuboid(Cuboid {
                    length: self.length,
                    width: self.width,
                    height: self.height,
                }),
                color: self.color,
            },
            Element {
                transform: Mat4::from_translation(Vec3::new(lc, -ls, 0.0))
                    * Mat4::from_angle_z(-cgmath::Rad(self.angle)),
                primitive: Primitive::Cuboid(Cuboid {
                    length: self.length,
                    width: self.width,
                    height: self.height,
                }),
                color: self.color,
            },
        ]
    }
}
