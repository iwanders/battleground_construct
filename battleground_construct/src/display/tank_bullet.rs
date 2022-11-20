use engine::prelude::*;
use super::primitives::*;

#[derive(Copy, Debug, Clone)]
pub struct TankBullet {
    pub radius: f32,
}

impl TankBullet {
    pub fn new() -> Self {
        TankBullet {
            radius: 0.1,
        }
    }
}
impl Component for TankBullet {}

impl Drawable for TankBullet {
    fn drawables(&self) -> Vec<Element> {
        vec![
            Element{
                transform:  Mat4::from_translation(Vec3::new(0.0, 0.0, 0.0)),
                primitive: Primitive::Sphere(Sphere{radius: self.radius}),
                color: Color{r: 30, g: 30, b: 30, a: 255},
            },
        ]
    }
}
