use super::primitives::*;
use engine::prelude::*;

// Tracks are cool... fixed velocity atm, but they destroy framerate.
const RENDER_TRACKS: bool = false;

#[derive(Copy, Debug, Clone)]
pub struct TankTracks {
    pub width: f32,
    pub length: f32,
    pub height: f32,
    pub track_width: f32,
    pub track_height: f32,
    pub epoch: std::time::Instant,
}

impl TankTracks {
    pub fn new() -> Self {
        TankTracks {
            width: 0.4,
            length: 1.4,
            height: 0.2,
            track_width: 1.0,
            track_height: 0.1,
            epoch: std::time::Instant::now(),
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
        let mut z = vec![
            Element {
                transform: Mat4::from_translation(Vec3::new(
                    0.0,
                    -self.track_width / 2.0,
                    self.track_height,
                )),
                primitive: track,
                color: color,
            },
            Element {
                transform: Mat4::from_translation(Vec3::new(
                    0.0,
                    self.track_width / 2.0,
                    self.track_height,
                )),
                primitive: track,
                color: color,
            },
        ];

        if RENDER_TRACKS {
            let t = self.epoch.elapsed().as_secs_f32();
            // Track length;
            let length = self.length;
            let height = self.height;
            let total_length = 2.0 * length + 2.0 * height;
            let bar_size = 0.05;
            let bar = Primitive::Cuboid(Cuboid {
                width: 0.5,
                height: bar_size,
                length: bar_size,
            });

            // Determine the track equation.
            let pos = |v: f32| {
                let v = v.rem_euclid(total_length);
                match v {
                    _ if 0.0 < v && v < length => {
                        // bottom section.
                        Mat4::from_translation(Vec3::new(
                            v - self.length / 2.0,
                            0.0,
                            -self.height / 2.0,
                        ))
                    }
                    _ if length < v && v < (length + height) => {
                        // front section.
                        Mat4::from_translation(Vec3::new(
                            self.length / 2.0,
                            0.0,
                            (v - length) - self.height / 2.0,
                        ))
                    }
                    _ if (length + height) < v && v < (length + height + length) => {
                        // top section.
                        Mat4::from_translation(Vec3::new(
                            self.length / 2.0 - (v - (length + height)),
                            0.0,
                            self.height / 2.0,
                        ))
                    }
                    _ if (length + height + length) < v
                        && v < (length + height + length + height) =>
                    {
                        // rear section.
                        Mat4::from_translation(Vec3::new(
                            -self.length / 2.0,
                            0.0,
                            self.height / 2.0 - (v - (length + height + length)),
                        ))
                    }
                    _ => Mat4::from_translation(Vec3::new(0.0, 0.0, 0.0)),
                }
            };
            let color = Color {
                r: 30,
                g: 30,
                b: 30,
                a: 255,
            };

            let v = 0.6;
            let offset = -v * t;
            let offset_normalized = offset.rem_euclid(total_length);

            let bars = 20;
            for i in 0..bars {
                let this_bar_pos = pos(i as f32 * (total_length / bars as f32) + offset_normalized);
                z.push(Element {
                    transform: this_bar_pos
                        * Mat4::from_translation(Vec3::new(
                            0.0,
                            self.track_width / 2.0,
                            self.track_height,
                        )),
                    primitive: bar,
                    color: color,
                });
                z.push(Element {
                    transform: this_bar_pos
                        * Mat4::from_translation(Vec3::new(
                            0.0,
                            -self.track_width / 2.0,
                            self.track_height,
                        )),
                    primitive: bar,
                    color: color,
                });
            }
        }
        z
    }
}
