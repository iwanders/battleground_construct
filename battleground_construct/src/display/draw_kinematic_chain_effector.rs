use super::primitives::*;
use crate::components;
use engine::prelude::*;

pub use battleground_unit_control::modules::draw::LineSegment;

#[derive(Debug, Clone, Default)]
pub struct DrawKinematicChainEffector {
    elements: Vec<Element>,
}

impl DrawKinematicChainEffector{
    pub fn update_cannon(
        &mut self,
        _cannon: &components::cannon::Cannon,
    ) {
        let width = 0.01;
        let m = Mat4::from_translation(Vec3::new(0.0, 0.0, 0.0));
        self.elements.clear();

        // Simple bar pointing out from the cannon.
        self.elements.push(Element {
            transform: m,
            primitive: Primitive::Line(Line {
                p0: (0.0, 0.0, 0.0),
                p1: (0.1, 0.0, 0.0),
                width,
            }),
            material: Material::OverlayMaterial(
                Color {
                    r: 255,
                    g: 0,
                    b: 0,
                    a: 255,
                }
                .into(),
            ),
        });
    }

    pub fn update_gun_battery(
        &mut self,
        guns: &components::gun_battery::GunBattery,
    ) {
        let width = 0.01;
        let m = Mat4::from_translation(Vec3::new(0.0, 0.0, 0.0));
        self.elements.clear();

        for pose in (0..guns.gun_count()).map(|i| guns.gun_pose(i)).flatten() {
            // Simple bar pointing out from the cannon.
            use crate::util::cgmath::{ToTranslation, ToHomogenous};
            let s = pose.to_translation();
            let e = (pose * Vec3::new(0.1, 0.0, 0.0).to_h()).to_translation();
            self.elements.push(Element {
                transform: m,
                primitive: Primitive::Line(Line {
                    p0: (s.x, s.y, s.z),
                    p1: (e.x, e.y, e.z),
                    width,
                }),
                material: Material::OverlayMaterial(
                    Color {
                        r: 255,
                        g: 0,
                        b: 0,
                        a: 255,
                    }
                    .into(),
                ),
            });
        }
    }


    pub fn update_radar(
        &mut self,
        radar: &components::radar::Radar,
    ) {
        let width = 0.01;
        let m = Mat4::from_translation(Vec3::new(0.0, 0.0, 0.0));
        self.elements.clear();
        let r = 0.2;
        let material = Material::OverlayMaterial(
                Color {
                    r: 0,
                    g: 0,
                    b: 255,
                    a: 255,
                }
                .into(),
            );

        let (yaw_y, yaw_x) = radar.detection_angle_yaw().sin_cos();
        self.elements.push(Element {
            transform: m,
            primitive: Primitive::Line(Line {
                p0: (0.0, 0.0, 0.0),
                p1: (yaw_x * r, yaw_y * r, 0.0),
                width,
            }),
            material,
        });
        self.elements.push(Element {
            transform: m,
            primitive: Primitive::Line(Line {
                p0: (0.0, 0.0, 0.0),
                p1: (yaw_x * r, -yaw_y * r, 0.0),
                width,
            }),
            material,
        });

        let r = 0.1;
        let (pitch_x, pitch_z) = radar.detection_angle_pitch().sin_cos();
        self.elements.push(Element {
            transform: m,
            primitive: Primitive::Line(Line {
                p0: (0.0, 0.0, 0.0),
                p1: (pitch_x * r, 0.0, pitch_z * r),
                width,
            }),
            material,
        });
        self.elements.push(Element {
            transform: m,
            primitive: Primitive::Line(Line {
                p0: (0.0, 0.0, 0.0),
                p1: (pitch_x * r, 0.0, -pitch_z * r),
                width,
            }),
            material,
        });

    }
}
impl Component for DrawKinematicChainEffector {}

impl Drawable for DrawKinematicChainEffector {
    fn drawables(&self) -> Vec<Element> {
        self.elements.clone()
    }
}
