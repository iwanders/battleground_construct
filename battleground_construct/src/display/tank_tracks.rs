use super::primitives::*;
use engine::prelude::*;

// Tracks are cool... and now feasible for the renderer!
const RENDER_TRACKS: bool = true;

#[derive(Copy, Debug, Clone)]
pub struct TankTracks {
    width: f32,
    length: f32,
    height: f32,
    track_width: f32,
    track_height: f32,

    left_distance: f32,
    right_distance: f32,
}
impl Default for TankTracks {
    fn default() -> Self {
        TankTracks::new()
    }
}

impl TankTracks {
    pub fn new() -> Self {
        TankTracks {
            width: 0.4,
            length: 1.4,
            height: 0.2,
            track_width: 1.0,
            track_height: 0.1,
            left_distance: 0.0,
            right_distance: 0.0,
        }
    }

    pub fn add_track_distance(&mut self, left_delta: f32, right_delta: f32) {
        let total_length = 2.0 * self.length + 2.0 * self.height;
        self.left_distance = (self.left_distance + left_delta).rem_euclid(total_length);
        self.right_distance = (self.right_distance + right_delta).rem_euclid(total_length);
    }
}
impl Component for TankTracks {}

impl Drawable for TankTracks {
    fn drawables(&self) -> Vec<Element> {
        let material: Material = Color {
            r: 20,
            g: 20,
            b: 20,
            a: 255,
        }
        .into();
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
                material,
            },
            Element {
                transform: Mat4::from_translation(Vec3::new(
                    0.0,
                    self.track_width / 2.0,
                    self.track_height,
                )),
                primitive: track,
                material,
            },
        ];

        if RENDER_TRACKS {
            // Track length;
            let length = self.length;
            let height = self.height;
            let total_length = 2.0 * length + 2.0 * height;
            let bar_size = 0.05;
            let bar_width = self.width + 0.1;
            let bar = Primitive::Cuboid(Cuboid {
                width: bar_width,
                height: bar_size,
                length: bar_size,
            });

            // Determine the track equation.
            let pos = |v: f32| {
                let v = v.rem_euclid(total_length);
                match v {
                    _ if 0.0 <= v && v < length => {
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
            let material: Material = Color {
                r: 50,
                g: 50,
                b: 50,
                a: 255,
            }
            .into();

            // Did the math in the wrong order... fix that here.
            let left_offset_normalized = total_length - self.left_distance;
            let right_offset_normalized = total_length - self.right_distance;

            let bars = 20;
            for i in 0..bars {
                z.push(Element {
                    transform: pos(i as f32 * (total_length / bars as f32) + left_offset_normalized)
                        * Mat4::from_translation(Vec3::new(
                            0.0,
                            self.track_width / 2.0,
                            self.track_height,
                        )),
                    primitive: bar,
                    material,
                });
                z.push(Element {
                    transform: pos(
                        i as f32 * (total_length / bars as f32) + right_offset_normalized
                    ) * Mat4::from_translation(Vec3::new(
                        0.0,
                        -self.track_width / 2.0,
                        self.track_height,
                    )),
                    primitive: bar,
                    material,
                });
            }
        }
        z
    }
}
