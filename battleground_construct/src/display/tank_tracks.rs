use super::primitives::*;
use engine::prelude::*;

#[derive(Copy, Debug, Clone)]
pub struct TankTracks {
    pub width: f32,
    pub length: f32,
    pub height: f32,
    pub track_width: f32,
    pub track_height: f32,
}

impl TankTracks {
    pub fn new() -> Self {
        TankTracks {
            width: 0.4,
            length: 1.4,
            height: 0.2,
            track_width: 1.0,
            track_height: 0.1,
        }
    }
}
impl Component for TankTracks {}

impl Drawable for TankTracks {
    fn drawables(&self) -> Vec<Element> {
        let color = Color {
                r: 20,
                g: 20,
                b: 20,
                a: 255,
            };
        let track = Primitive::Cuboid(Cuboid {
                width: self.width,
                height: self.height,
                length: self.length,
            });
        vec![Element {
            transform: Mat4::from_translation(Vec3::new(0.0, -self.track_width / 2.0, self.track_height)),
            primitive: track,
            color: color,
        }, Element {
            transform: Mat4::from_translation(Vec3::new(0.0, self.track_width / 2.0, self.track_height)),
            primitive: track,
            color: color,
        }]
    }
}
