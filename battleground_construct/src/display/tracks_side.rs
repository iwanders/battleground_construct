use super::primitives::*;
use crate::components::hit_box::HitBox;
use engine::prelude::*;

// Tracks are cool... and now feasible for the renderer!
const RENDER_TRACKS: bool = true;

#[derive(Copy, Debug, Clone)]
pub struct TracksSideConfig {
    /// Width of an individual track.
    pub width: f32,
    /// Length of an individual track.
    pub length: f32,
    /// Height of an individual track.
    pub height: f32,

    /// The distance between the tracks left to right.
    pub track_width: f32,
}

#[derive(Copy, Debug, Clone)]
pub struct TracksSide {
    config: TracksSideConfig,

    /// Z offset of tracks, usually half of the height.
    track_height: f32,

    /// Distance travelled of the left track.
    left_distance: f32,
    /// Distance travelled of the right track.
    right_distance: f32,

    /// The entity id to track for display.
    diff_drive_entity: EntityId,
}

impl TracksSide {
    pub fn from_config(config: TracksSideConfig, diff_drive_entity: EntityId) -> Self {
        TracksSide {
            config,
            track_height: config.height / 2.0,
            left_distance: 0.0,
            right_distance: 0.0,
            diff_drive_entity,
        }
    }

    pub fn diff_drive_entity(&self) -> EntityId {
        self.diff_drive_entity
    }

    pub fn add_track_distance(&mut self, left_delta: f32, right_delta: f32) {
        let total_length = 2.0 * self.config.length + 2.0 * self.config.height;
        self.left_distance = (self.left_distance + left_delta).rem_euclid(total_length);
        self.right_distance = (self.right_distance + right_delta).rem_euclid(total_length);
    }

    pub fn hit_boxes(&self) -> Vec<(Mat4, HitBox)> {
        let track = HitBox::new(self.config.length, self.config.width, self.config.height);
        vec![
            (
                Mat4::from_translation(Vec3::new(
                    0.0,
                    -self.config.track_width / 2.0,
                    self.track_height,
                )),
                track,
            ),
            (
                Mat4::from_translation(Vec3::new(
                    0.0,
                    self.config.track_width / 2.0,
                    self.track_height,
                )),
                track,
            ),
        ]
    }
}
impl Component for TracksSide {}

impl Drawable for TracksSide {
    fn drawables(&self) -> Vec<Element> {
        let material: Material = Color {
            r: 20,
            g: 20,
            b: 20,
            a: 255,
        }
        .into();

        let mut z = vec![];

        for (t, b) in self.hit_boxes() {
            let track = Primitive::Cuboid(Cuboid {
                width: b.width(),
                height: b.height(),
                length: b.length(),
            });

            z.push(Element {
                transform: t,
                primitive: track,
                material,
            });
        }

        if RENDER_TRACKS {
            // Track length;
            let length = self.config.length;
            let height = self.config.height;
            let total_length = 2.0 * length + 2.0 * height;
            let bar_size = 0.05;
            let bar_width = self.config.width + 0.1;
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
                            v - self.config.length / 2.0,
                            0.0,
                            -self.config.height / 2.0,
                        ))
                    }
                    _ if length < v && v < (length + height) => {
                        // front section.
                        Mat4::from_translation(Vec3::new(
                            self.config.length / 2.0,
                            0.0,
                            (v - length) - self.config.height / 2.0,
                        ))
                    }
                    _ if (length + height) < v && v < (length + height + length) => {
                        // top section.
                        Mat4::from_translation(Vec3::new(
                            self.config.length / 2.0 - (v - (length + height)),
                            0.0,
                            self.config.height / 2.0,
                        ))
                    }
                    _ if (length + height + length) < v
                        && v < (length + height + length + height) =>
                    {
                        // rear section.
                        Mat4::from_translation(Vec3::new(
                            -self.config.length / 2.0,
                            0.0,
                            self.config.height / 2.0 - (v - (length + height + length)),
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
                            self.config.track_width / 2.0,
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
                        -self.config.track_width / 2.0,
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
