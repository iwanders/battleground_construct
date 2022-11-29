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
            transform: Mat4::from_translation(Vec3::new(0.0, 0.0, 1.0)),
            primitive: Primitive::Cuboid(Cuboid {
                width: 1.0,
                height: 1.0,
                length: 1.0,
            }),
            color: Color::RED,
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
