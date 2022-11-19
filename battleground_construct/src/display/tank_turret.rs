use engine::prelude::*;
use super::primitives::*;

#[derive(Copy, Debug, Clone)]
pub struct TankTurret {
    pub width: f32,
    pub length: f32,
    pub height: f32,
    pub z_offset: f32,
}

impl TankTurret {
    pub fn new() -> Self {
        TankTurret {
            width: 0.5,
            length: 0.7,
            height: 0.1,
            z_offset: 0.0,
        }
    }
}
impl Component for TankTurret {}

impl Drawable for TankTurret {
    fn drawables(&self) -> Vec<Element> {
        vec![
            Element{
                transform:  Mat4::from_translation(Vec3::new(0.0, 0.0, self.z_offset)),
                primitive: Primitive::Cuboid(Cuboid{width: self.width, height: self.height, length: self.length}),
                color: Color{r: 255, g: 191, b: 0, a: 255},
            },
        ]
    }
}
