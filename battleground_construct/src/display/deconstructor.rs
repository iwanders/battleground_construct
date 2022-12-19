use super::primitives::*;
use engine::prelude::*;

#[derive(Debug, Clone)]
pub struct Deconstructor {
    pub entity: EntityId,
    pub elements: Vec<(Element, Twist, Mat4)>,
    pub impacts: Vec<(Mat4, f32)>,
}

impl Deconstructor {
    pub fn new(entity: EntityId) -> Self {
        Deconstructor {
            entity,
            elements: vec![],
            impacts: vec![],
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
            let world_vel = crate::components::velocity::world_velocity(world, entity).to_twist();
            for el in component.drawables() {
                self.elements.push((el, world_vel, *world_pose))
            }
        }
    }

    pub fn add_impact(&mut self, impact: Mat4, magnitude: f32) {
        self.impacts.push((impact, magnitude));
    }
}
impl Component for Deconstructor {}

impl Drawable for Deconstructor {
    fn effects(&self) -> Vec<Effect> {
        vec![Effect {
            id: (self.entity, 0),
            effect: EffectType::Deconstructor {
                elements: self.elements.clone(),
                impacts: self.impacts.clone(),
            },
            transform: Mat4::from_translation(Vec3::new(0.0, 0.0, 0.0)),
        }]
    }
}
