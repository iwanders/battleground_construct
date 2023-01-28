use super::primitives::*;
use crate::components;
use engine::prelude::*;

pub use battleground_unit_control::modules::draw::LineSegment;

#[derive(Debug, Clone, Default)]
pub struct DrawKinematicChainRevolute {
    elements: Vec<Element>,
}

impl DrawKinematicChainRevolute {
    pub fn update(
        &mut self,
        pre_transform: Option<&components::pose::PreTransform>,
        revolute: &components::revolute::Revolute,
    ) {
        let width = 0.05;
        let mut m = Mat4::from_translation(Vec3::new(0.0, 0.0, 0.0));
        self.elements.clear();
        println!("Update");

        if let Some(p) = pre_transform {
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
            m = *p.transform();
        }

        // now, we want to draw a circle at m, in the plane of the revolute direction.
        fn add_ricle(elements: &mut Vec<Element>, transform: Mat4, r: f32, width: f32) {
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
        add_ricle(&mut self.elements, m, 0.5, width);
    }
}
impl Component for DrawKinematicChainRevolute {}

impl Drawable for DrawKinematicChainRevolute {
    fn drawables(&self) -> Vec<Element> {
        self.elements.clone()
    }
}
