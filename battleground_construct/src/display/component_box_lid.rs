use super::primitives::*;
use engine::prelude::*;

const WALL_THICKNESS: f32 = 0.02;

#[derive(Copy, Debug, Clone)]
pub struct ComponentBoxLid {
    pub width: f32,
    pub length: f32,
}

impl ComponentBoxLid {
    pub fn from_config(config: crate::units::common::ComponentBoxSpawnConfig) -> Self {
        ComponentBoxLid {
            width: config.width / 2.0,
            length: config.length,
        }
    }
    pub fn lid_offset(&self) -> Vec3 {
        Vec3::new(0.0, -self.width, 0.0)
    }
}
impl Component for ComponentBoxLid {}

impl Drawable for ComponentBoxLid {
    fn drawables(&self) -> Vec<Element> {
        let material: Material = Color {
            r: 200,
            g: 100,
            b: 0,
            a: 255,
        }
        .into();

        vec![Element {
            transform: Mat4::from_translation(Vec3::new(self.length / 2.0, -self.width / 2.0, 0.0)),
            primitive: Primitive::Cuboid(Cuboid {
                width: self.width,
                height: WALL_THICKNESS,
                length: self.length - WALL_THICKNESS,
            }),
            material,
        }]
    }
}
