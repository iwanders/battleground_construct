use super::primitives::*;
use crate::components;
use crate::util::cgmath::prelude::*;
use engine::prelude::*;

pub use battleground_unit_control::modules::draw::LineSegment;

#[derive(Debug, Clone, Default)]
pub struct DrawKinematicChainLink {
    elements: Vec<Element>,
}

impl DrawKinematicChainLink {
    pub fn update(
        &mut self,
        pose: &components::pose::Pose,
        pre_transform: Option<&components::pose::PreTransform>,
        revolute: Option<&components::revolute::Revolute>,
    ) {
        let width = 0.05;
        let mut m = Mat4::from_translation(Vec3::new(0.0, 0.0, 0.0));
        self.elements.clear();

        // if let Some(p) = pose {
        m = m * pose.transform().to_inv_h();
        // }

        if let Some(p) = pre_transform {
            m = m * p.transform().to_inv_h();
            // }

            self.elements.push(Element {
                transform: m,
                primitive: Primitive::Line(Line {
                    p0: (0.0, 0.0, 0.0),
                    p1: (p.w.x, p.w.y, p.w.z),
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
            // m = p.transform().to_inv_h();
            m = Mat4::from_translation(Vec3::new(0.0, 0.0, 0.0));
        }

        // now, we want to draw a circle at m, in the plane of the revolute direction.
        fn add_circle(elements: &mut Vec<Element>, transform: Mat4, r: f32, width: f32) {
            let max = 31;
            for i in 1..max {
                let prev = i - 1;
                let pp = (prev as f32 / (max - 1) as f32) * std::f32::consts::PI * 2.0;
                let pn = (i as f32 / (max - 1) as f32) * std::f32::consts::PI * 2.0;
                let (px, py) = pp.sin_cos();
                let (nx, ny) = pn.sin_cos();
                elements.push(Element {
                    transform,
                    primitive: Primitive::Line(Line {
                        p0: (px * r, py * r, 0.0),
                        p1: (nx * r, ny * r, 0.0),
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
        if let Some(revolute) = revolute {
            let joint_ortho = m * Vec3::new(0.0, 0.0, 1.0)
                .rotation_from(revolute.axis())
                .to_h();
            add_circle(&mut self.elements, joint_ortho, 0.10, width * 0.2);
        }
    }
}
impl Component for DrawKinematicChainLink {}

impl Drawable for DrawKinematicChainLink {
    fn drawables(&self) -> Vec<Element> {
        self.elements.clone()
    }
}