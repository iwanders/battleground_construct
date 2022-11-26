use engine::prelude::*;
use crate::components::pose::Pose;

// This must be an Rc, as wel need to be able to copy it to allow a mutable world, we cannot borrow
// it out of the cannon.
pub type CannonFireEffect = std::rc::Rc<dyn for<'a> Fn(&'a mut World, &Pose, &EntityId) -> ()>;

pub struct CannonConfig {
    pub reload_time: f32,
    pub fire_effect: CannonFireEffect,
}

#[derive()]
pub struct Cannon {
    reload_time: f32,
    last_fire_time: f32,
    config: CannonConfig,
}

impl Cannon {
    pub fn new(config: CannonConfig) -> Self {
        Self {
            reload_time: 2.0,
            last_fire_time: -2.0, // spawn ready to fire.
            config,
        }
    }
    pub fn is_ready(&self, current_time: f32) -> bool {
        (current_time - self.last_fire_time) > self.reload_time
    }
    pub fn fire(&mut self, current_time: f32) {
        self.last_fire_time = current_time;
    }

    pub fn effect(&self) -> CannonFireEffect {
        self.config.fire_effect.clone()
    }
}
impl Component for Cannon {}
