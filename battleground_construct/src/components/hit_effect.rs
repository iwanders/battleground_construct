use crate::components;
use engine::prelude::*;

pub type HitEffectFn = std::rc::Rc<
    dyn for<'a> Fn(&'a mut World, /*projectile*/ EntityId, &components::impact::Impact),
>;

#[derive(Clone)]
pub struct HitEffect {
    effect: HitEffectFn,
}

impl HitEffect {
    pub fn new(effect: HitEffectFn) -> Self {
        HitEffect { effect }
    }

    pub fn effect(&self) -> HitEffectFn {
        self.effect.clone()
    }
}
impl Component for HitEffect {}
