use super::primitives::*;
use engine::prelude::*;

#[derive(Copy, Debug, Clone)]
pub struct WheeledSteerBeam {}

impl Component for WheeledSteerBeam {}

impl Drawable for WheeledSteerBeam {
    fn drawables(&self) -> Vec<Element> {
        let material: Material = Color {
            r: 20,
            g: 20,
            b: 20,
            a: 255,
        }
        .into();
        vec![Element {
            transform: Mat4::from_translation(Vec3::new(0.15, 0.0, 0.0)),
            primitive: Primitive::Cuboid(Cuboid {
                length: 0.3,
                width: 0.1,
                height: 0.1,
            }),
            material,
        }]
    }
}
