use super::primitives::*;
use crate::components::hit_box::HitBox;
use crate::components::pose::Pose;
use engine::prelude::*;

const WHEELED_BODY_HEIGHT: f32 = 0.15;
const WHEELED_BODY_WIDTH: f32 = 1.0;
const WHEELED_BODY_LENGTH: f32 = 1.5;
const WHEELED_BODY_ORIGIN_SHIFT: f32 = 0.5;
const WHEELED_BODY_AXLE_PROTRUSION: f32 = 0.1;
const WHEELED_BODY_AXLE_RADIUS: f32 = 0.05;
const WHEELED_BODY_AXLE_OFFSET: f32 = -WHEELED_BODY_HEIGHT / 2.0 - WHEELED_BODY_AXLE_RADIUS / 2.0;
const WHEELED_BODY_WHEELBASE: f32 = 1.0;

#[derive(Copy, Debug, Clone)]
pub struct WheeledBody {
    pub length: f32,
    pub width: f32,
    pub height: f32,
    color: Color,
}
impl Default for WheeledBody {
    fn default() -> Self {
        WheeledBody::new()
    }
}

impl WheeledBody {
    pub fn new() -> Self {
        WheeledBody {
            length: WHEELED_BODY_LENGTH,
            width: WHEELED_BODY_WIDTH,
            height: WHEELED_BODY_HEIGHT,
            color: Color {
                r: 0,
                g: 20,
                b: 0,
                a: 255,
            },
        }
    }
    pub fn set_color(&mut self, color: Color) {
        self.color = color;
    }

    pub fn hitbox(&self) -> HitBox {
        HitBox::new(self.length, self.width, self.height)
    }

    pub fn pose_rear_left_wheel(&self) -> Pose {
        Pose::from_translation(Vec3::new(
            0.0,
            -(WHEELED_BODY_WIDTH + WHEELED_BODY_AXLE_PROTRUSION * 2.0) / 2.0,
            WHEELED_BODY_AXLE_OFFSET,
        ))
        .rotated_angle_z(cgmath::Deg(90.0))
    }
    pub fn pose_rear_right_wheel(&self) -> Pose {
        Pose::from_translation(Vec3::new(
            0.0,
            (WHEELED_BODY_WIDTH + WHEELED_BODY_AXLE_PROTRUSION * 2.0) / 2.0,
            WHEELED_BODY_AXLE_OFFSET,
        ))
        .rotated_angle_z(cgmath::Deg(90.0))
    }

    pub fn pose_front_left_wheel(&self) -> Pose {
        Pose::from_translation(Vec3::new(
            WHEELED_BODY_WHEELBASE,
            -(WHEELED_BODY_WIDTH + WHEELED_BODY_AXLE_PROTRUSION * 2.0) / 2.0,
            WHEELED_BODY_AXLE_OFFSET,
        ))
        .rotated_angle_z(cgmath::Deg(90.0))
    }
    pub fn pose_front_right_wheel(&self) -> Pose {
        Pose::from_translation(Vec3::new(
            WHEELED_BODY_WHEELBASE,
            (WHEELED_BODY_WIDTH + WHEELED_BODY_AXLE_PROTRUSION * 2.0) / 2.0,
            WHEELED_BODY_AXLE_OFFSET,
        ))
        .rotated_angle_z(cgmath::Deg(90.0))
    }

    pub fn track_width(&self) -> f32 {
        WHEELED_BODY_WIDTH + WHEELED_BODY_AXLE_PROTRUSION * 2.0
    }
}
impl Component for WheeledBody {}

impl Drawable for WheeledBody {
    fn drawables(&self) -> Vec<Element> {
        let addition = Color::rgb(10, 10, 10);
        let emissive_material = Material::FlatMaterial(FlatMaterial {
            color: self.color, // .saturating_add(&addition)
            is_emissive: true,
            emissive: self.color.saturating_add(&addition),
            ..Default::default()
        });

        vec![
            // main body
            Element {
                transform: Mat4::from_translation(Vec3::new(WHEELED_BODY_ORIGIN_SHIFT, 0.0, 0.0)),
                primitive: Primitive::Cuboid(Cuboid {
                    length: self.length,
                    width: self.width,
                    height: self.height,
                }),
                material: self.color.into(),
            },
            // rear axle (at origin);
            Element {
                transform: Mat4::from_translation(Vec3::new(0.0, 0.0, WHEELED_BODY_AXLE_OFFSET)),
                primitive: Primitive::Cuboid(Cuboid {
                    length: WHEELED_BODY_AXLE_RADIUS,
                    width: self.width + WHEELED_BODY_AXLE_PROTRUSION * 2.0,
                    height: WHEELED_BODY_AXLE_RADIUS,
                }),
                material: self.color.into(),
            },
            // front axle (at origin);
            Element {
                transform: Mat4::from_translation(Vec3::new(
                    WHEELED_BODY_WHEELBASE,
                    0.0,
                    WHEELED_BODY_AXLE_OFFSET,
                )),
                primitive: Primitive::Cuboid(Cuboid {
                    length: WHEELED_BODY_AXLE_RADIUS,
                    width: self.width + WHEELED_BODY_AXLE_PROTRUSION * 2.0,
                    height: WHEELED_BODY_AXLE_RADIUS,
                }),
                material: self.color.into(),
            },
            Element {
                transform: Mat4::from_translation(Vec3::new(
                    WHEELED_BODY_ORIGIN_SHIFT,
                    0.4,
                    WHEELED_BODY_HEIGHT / 2.0 + 0.01,
                )),
                primitive: Primitive::Cuboid(Cuboid {
                    length: self.length - 0.25,
                    width: 0.1,
                    height: 0.02,
                }),
                material: emissive_material,
            },
            Element {
                transform: Mat4::from_translation(Vec3::new(
                    WHEELED_BODY_ORIGIN_SHIFT,
                    -0.4,
                    WHEELED_BODY_HEIGHT / 2.0 + 0.01,
                )),
                primitive: Primitive::Cuboid(Cuboid {
                    length: self.length - 0.25,
                    width: 0.1,
                    height: 0.02,
                }),
                material: emissive_material,
            },
        ]
    }
}
