use super::primitives::*;
use engine::prelude::*;

#[derive(Copy, Debug, Clone)]
pub struct TankBullet {
    pub radius: f32,
}
impl Default for TankBullet {
    fn default() -> Self {
        TankBullet::new()
    }
}

impl TankBullet {
    pub fn new() -> Self {
        TankBullet { radius: 0.05 }
    }
}
impl Component for TankBullet {}

impl Drawable for TankBullet {
    fn drawables(&self) -> Vec<Element> {
        vec![Element {
            transform: Mat4::from_translation(Vec3::new(0.0, 0.0, 0.0)),
            primitive: Primitive::Sphere(Sphere {
                radius: self.radius,
            }),
            material: Color {
                r: 20,
                g: 20,
                b: 20,
                a: 255,
            }
            .into(),
        }]
    }
}
