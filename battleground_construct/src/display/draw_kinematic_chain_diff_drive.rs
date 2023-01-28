use super::primitives::*;
use crate::components;
use engine::prelude::*;

pub use battleground_unit_control::modules::draw::LineSegment;

#[derive(Debug, Clone, Default)]
pub struct DrawKinematicChainDiffDrive {
    elements: Vec<Element>,
}

impl DrawKinematicChainDiffDrive {
    pub fn update(
        &mut self,
        diff_drive: &components::differential_drive_base::DifferentialDriveBase,
    ) {
        let track_width = diff_drive.track_width();
        let wheel_line_length = track_width / 4.0;
        let width = 0.025;
        let m = Mat4::from_translation(Vec3::new(0.0, 0.0, 0.0));
        self.elements.clear();

        // bar between the wheels.
        self.elements.push(Element {
            transform: m,
            primitive: Primitive::Line(Line {
                p0: (0.0, -track_width / 2.0, 0.0),
                p1: (0.0, track_width / 2.0, 0.0),
                width,
            }),
            material: Material::OverlayMaterial(
                Color {
                    r: 255,
                    g: 255,
                    b: 0,
                    a: 255,
                }
                .into(),
            ),
        });

        self.elements.push(Element {
            transform: m,
            primitive: Primitive::Line(Line {
                p0: (wheel_line_length / 2.0, -track_width / 2.0, 0.0),
                p1: (-wheel_line_length / 2.0, -track_width / 2.0, 0.0),
                width,
            }),
            material: Material::OverlayMaterial(
                Color {
                    r: 255,
                    g: 255,
                    b: 0,
                    a: 255,
                }
                .into(),
            ),
        });
        self.elements.push(Element {
            transform: m,
            primitive: Primitive::Line(Line {
                p0: (wheel_line_length / 2.0, track_width / 2.0, 0.0),
                p1: (-wheel_line_length / 2.0, track_width / 2.0, 0.0),
                width,
            }),
            material: Material::OverlayMaterial(
                Color {
                    r: 255,
                    g: 255,
                    b: 0,
                    a: 255,
                }
                .into(),
            ),
        });
    }
}
impl Component for DrawKinematicChainDiffDrive {}

impl Drawable for DrawKinematicChainDiffDrive {
    fn drawables(&self) -> Vec<Element> {
        self.elements.clone()
    }
}
