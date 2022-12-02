use super::primitives::*;
use engine::prelude::*;

#[derive(Debug, Clone)]
pub struct Deconstructor {
    pub entity: EntityId,
    pub elements: Vec<Element>,
}

impl Deconstructor {
    pub fn new(entity: EntityId) -> Self {
        Deconstructor {
            entity,
            elements: vec![Element {
            transform: Mat4::from_translation(Vec3::new(0.0, 0.0, 1.0)) * Mat4::from_angle_y(cgmath::Deg(15.0)),
            primitive: Primitive::Cuboid(Cuboid {
                width: 1.0,
                height: 1.0,
                length: 1.0,
            }),
            color: Color::RED,
        }, Element {
            transform: Mat4::from_translation(Vec3::new(-2.0, 0.0, 1.0)) * Mat4::from_angle_x(cgmath::Deg(45.0)),
            primitive: Primitive::Cuboid(Cuboid {
                width: 1.51,
                height: 0.32,
                length: 0.24,
            }),
            color: Color::BLUE,
        }],
        }
    }
}
impl Component for Deconstructor {}

impl Drawable for Deconstructor {
    fn effects(&self) -> Vec<Effect> {
        vec![Effect {
            id: (self.entity, 0),
            effect: EffectType::Deconstructor {
                elements: self.elements.clone()
            },
            transform: Mat4::from_translation(Vec3::new(0.0, 0.0, 0.0)),
        }]
    }
}
