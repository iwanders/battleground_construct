use super::primitives::*;
use crate::components::hit_box::HitBox;
use engine::prelude::*;

const RENDER_TRACKS: bool = true;

#[derive(Copy, Debug, Clone)]
pub struct WheelConfig {
    /// Width of an individual wheel.
    pub width: f32,
    /// Radius of an individual wheel.
    pub radius: f32,
}

#[derive(Copy, Debug, Clone)]
pub struct Wheel {
    config: WheelConfig,

    /// Distance travelled of this wheel.
    distance: f32,
}

impl Wheel {
    pub fn from_config(config: WheelConfig) -> Self {
        Wheel {
            config,
            distance: 0.0,
        }
    }

    pub fn add_track_distance(&mut self, delta: f32) {
        let total_length = 2.0 * self.config.radius * std::f32::consts::PI;
        self.distance = (self.distance + delta).rem_euclid(total_length);
    }

    pub fn hit_boxes(&self) -> Vec<(Mat4, HitBox)> {
        let track = HitBox::new(self.config.radius, self.config.width, self.config.radius);
        vec![(
            Mat4::from_translation(Vec3::new(0.0, -self.config.width / 2.0, self.config.radius)),
            track,
        )]
    }
}
impl Component for Wheel {}

impl Drawable for Wheel {
    fn drawables(&self) -> Vec<Element> {
        let material: Material = Color {
            r: 20,
            g: 20,
            b: 20,
            a: 255,
        }
        .into();

        let c = -self.config.width / 2.0;

        let mut r = vec![
            // Main cylinder
            Element {
                transform: Mat4::from_translation(Vec3::new(c, 0.0, 0.0)),
                primitive: Primitive::Cylinder(Cylinder {
                    height: self.config.width,
                    radius: self.config.radius,
                }),
                material,
            },
            // Cap at the base.
            Element {
                transform: Mat4::from_translation(Vec3::new(c, 0.0, 0.0))
                    * Mat4::from_angle_y(cgmath::Deg(90.0)),
                primitive: Primitive::Circle(Circle {
                    radius: self.config.radius,
                }),
                material,
            },
            // Cap at the end.
            Element {
                transform: Mat4::from_translation(Vec3::new(c + self.config.width, 0.0, 0.0))
                    * Mat4::from_angle_y(cgmath::Deg(90.0)),
                primitive: Primitive::Circle(Circle {
                    radius: self.config.radius,
                }),
                material,
            },
        ];

        if RENDER_TRACKS {
            // Track length;
            let total_length = self.config.radius * 2.0 * std::f32::consts::PI;
            let bar_size = self.config.radius * 0.15;
            let bar_width = self.config.width + self.config.width * 0.25;
            let bar = Primitive::Cuboid(Cuboid {
                width: bar_size,
                height: bar_size,
                length: bar_width,
            });

            // Determine the track equation.
            let pos = |v: f32| {
                let v = v.rem_euclid(total_length);
                Mat4::from_angle_x(cgmath::Rad(v * 2.0 * std::f32::consts::PI))
                    * Mat4::from_translation(Vec3::new(
                        c + self.config.width / 2.0,
                        0.0,
                        self.config.radius,
                    ))
            };
            let material: Material = Color {
                r: 50,
                g: 50,
                b: 50,
                a: 255,
            }
            .into();

            let bars = 15;
            for i in 0..=bars {
                r.push(Element {
                    transform: pos(i as f32 * (total_length / bars as f32) + self.distance),
                    primitive: bar,
                    material,
                });
            }
        }

        r
    }
}
