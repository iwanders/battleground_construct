use super::primitives::*;
use crate::components::hit_box::HitBox;
use crate::components::pose::Pose;
use engine::prelude::*;

pub const WHEELED_BODY_HEIGHT: f32 = 0.20;
pub const WHEELED_BODY_WIDTH: f32 = 1.0;
pub const WHEELED_BODY_LENGTH: f32 = 1.5;
pub const WHEELED_BODY_ORIGIN_SHIFT: f32 = 0.5;
pub const WHEELED_BODY_ORIGIN_Z: f32 =
    battleground_unit_control::units::base_tricycle::BASE_TRICYCLE_DIM_FLOOR_TO_BODY_Z;
pub const WHEELED_BODY_AXLE_PROTRUSION: f32 = 0.1;
pub const WHEELED_BODY_AXLE_RADIUS: f32 = 0.05;
pub const WHEELED_BODY_AXLE_OFFSET: f32 = -WHEELED_BODY_HEIGHT / 2.0;
pub const WHEELED_BODY_WHEELBASE: f32 = 1.5;
pub const WHEELED_BODY_WHEEL_RADIUS: f32 = 0.15;

pub const WHEELED_BODY_CABIN_LENGTH: f32 = 0.3;
pub const WHEELED_BODY_CABIN_MARGIN: f32 = 0.05;
pub const WHEELED_BODY_CABIN_HEIGHT: f32 = 0.4;
pub const WHEELED_BODY_CABIN_WIDTH: f32 = WHEELED_BODY_WIDTH - WHEELED_BODY_CABIN_MARGIN;
pub const WHEELED_BODY_WINDOW_HEIGHT: f32 = 0.15;
pub const WHEELED_BODY_FRONT_PLATFORM: f32 = 0.4;

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
        HitBox::new(
            WHEELED_BODY_LENGTH + WHEELED_BODY_FRONT_PLATFORM,
            self.width,
            self.height,
        )
    }

    pub fn center_offset(&self) -> f32 {
        WHEELED_BODY_WHEELBASE / 2.0 + WHEELED_BODY_AXLE_OFFSET + WHEELED_BODY_CABIN_MARGIN
    }

    pub fn cabin_offset(&self) -> Vec3 {
        Vec3::new(
            WHEELED_BODY_WHEELBASE - WHEELED_BODY_CABIN_MARGIN / 2.0,
            0.0,
            WHEELED_BODY_CABIN_HEIGHT / 2.0 + WHEELED_BODY_HEIGHT / 2.0,
        )
    }
    pub fn payload_offset(&self) -> Vec3 {
        Vec3::new(
            (WHEELED_BODY_LENGTH - WHEELED_BODY_ORIGIN_SHIFT) / 2.0,
            0.0,
            WHEELED_BODY_WHEEL_RADIUS + WHEELED_BODY_HEIGHT,
        )
    }
    pub fn cabin_hitbox(&self) -> HitBox {
        HitBox::new(
            WHEELED_BODY_CABIN_LENGTH,
            WHEELED_BODY_CABIN_WIDTH,
            WHEELED_BODY_CABIN_HEIGHT,
        )
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
            0.0,
            WHEELED_BODY_AXLE_OFFSET,
        ))
        .rotated_angle_z(cgmath::Deg(90.0))
    }
    pub fn pose_front_right_wheel(&self) -> Pose {
        Pose::from_translation(Vec3::new(
            WHEELED_BODY_WHEELBASE,
            0.0,
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
            // front steer rotation
            Element {
                transform: Mat4::from_translation(Vec3::new(WHEELED_BODY_WHEELBASE, 0.0, -0.1))
                    * Mat4::from_angle_y(cgmath::Deg(-90.0)),
                primitive: Primitive::Cylinder(Cylinder {
                    height: 0.20,
                    radius: 0.05,
                }),
                material: self.color.into(),
            },
            // front platform above steering.
            Element {
                transform: Mat4::from_translation(Vec3::new(
                    0.1 + WHEELED_BODY_WHEELBASE - /* length */ 0.3 / 2.0,
                    0.0,
                    WHEELED_BODY_HEIGHT / 2.0  - /*height*/ 0.04 / 2.0,
                )),
                primitive: Primitive::Cuboid(Cuboid {
                    length: WHEELED_BODY_FRONT_PLATFORM,
                    width: WHEELED_BODY_WIDTH,
                    height: 0.04,
                }),
                material: self.color.into(),
            },
            // The 'cabin'
            Element {
                transform: Mat4::from_translation(Vec3::new(
                    WHEELED_BODY_WHEELBASE - WHEELED_BODY_CABIN_MARGIN / 2.0,
                    0.0,
                    WHEELED_BODY_HEIGHT / 2.0 + WHEELED_BODY_CABIN_HEIGHT / 2.0,
                )),
                primitive: Primitive::Cuboid(Cuboid {
                    length: WHEELED_BODY_CABIN_LENGTH,
                    width: self.width - WHEELED_BODY_CABIN_MARGIN,
                    height: WHEELED_BODY_CABIN_HEIGHT,
                }),
                material: self.color.into(),
            },
            // emissive team color, use as 'window'
            Element {
                transform: Mat4::from_translation(Vec3::new(
                    WHEELED_BODY_WHEELBASE - WHEELED_BODY_CABIN_MARGIN / 2.0
                        + WHEELED_BODY_CABIN_LENGTH / 2.0
                        + 0.01,
                    0.0,
                    WHEELED_BODY_HEIGHT / 2.0 + WHEELED_BODY_CABIN_HEIGHT
                        - WHEELED_BODY_WINDOW_HEIGHT / 2.0
                        - WHEELED_BODY_CABIN_MARGIN,
                )),
                primitive: Primitive::Cuboid(Cuboid {
                    length: 0.02,
                    width: self.width - WHEELED_BODY_CABIN_MARGIN * 2.0,
                    height: WHEELED_BODY_WINDOW_HEIGHT,
                }),
                material: emissive_material,
            },
        ]
    }
}
