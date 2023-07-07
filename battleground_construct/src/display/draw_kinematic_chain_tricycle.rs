use super::primitives::*;
use crate::components;
use engine::prelude::*;

pub use battleground_unit_control::modules::draw::LineSegment;

#[derive(Debug, Clone, Default)]
pub struct DrawKinematicChainTricycle {
    elements: Vec<Element>,
}

impl DrawKinematicChainTricycle {
    pub fn update(&mut self, base: &components::tricycle_base::TricycleBase) {
        let width = 0.025;
        let m = Mat4::from_translation(Vec3::new(0.0, 0.0, 0.0));
        self.elements.clear();

        let material = Material::OverlayMaterial(
            Color {
                r: 128,
                g: 128,
                b: 128,
                a: 255,
            }
            .into(),
        );
        // bar between the wheels.
        self.elements.push(Element {
            transform: m,
            primitive: Primitive::Line(Line {
                p0: (0.0, 0.0, 0.0),
                p1: (base.wheel_base(), 0.0, 0.0),
                width,
            }),
            material,
        });
    }
}
impl Component for DrawKinematicChainTricycle {}

impl Drawable for DrawKinematicChainTricycle {
    fn drawables(&self) -> Vec<Element> {
        self.elements.clone()
    }
}
