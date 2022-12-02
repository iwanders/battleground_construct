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
            elements: vec![],
        }
    }

    pub fn add_element<C: Component + Drawable + 'static>(
        &mut self,
        entity: EntityId,
        world: &World,
    ) {
        if let Some(component) = world.component::<C>(entity) {
            // Get the world pose for this entity, to add draw transform local to this component.
            // let world_pose = construct.entity_pose(entity);
            let world_pose = crate::components::pose::world_pose(world, entity);
            for el in component.drawables() {
                let mut el = el;
                el.transform = world_pose.transform() * el.transform;
                self.elements.push(el)
            }
        }
    }
}
impl Component for Deconstructor {}

impl Drawable for Deconstructor {
    fn effects(&self) -> Vec<Effect> {
        vec![Effect {
            id: (self.entity, 0),
            effect: EffectType::Deconstructor {
                elements: self.elements.clone(),
            },
            transform: Mat4::from_translation(Vec3::new(0.0, 0.0, 0.0)),
        }]
    }
}
