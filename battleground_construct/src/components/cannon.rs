use engine::prelude::*;

// This must be an Rc, as wel need to be able to copy it to allow a mutable world, we cannot borrow
// it out of the cannon.
pub type CannonFireEffect = std::rc::Rc<dyn for<'a> Fn(&'a mut World, EntityId)>;

pub struct CannonConfig {
    pub reload_time: f32,
    pub fire_effect: CannonFireEffect,
}

#[derive()]
pub struct Cannon {
    reload_time: f32,
    last_fire_time: f32,
    is_ready: bool,
    is_firing: bool,
    config: CannonConfig,
}

impl Cannon {
    pub fn new(config: CannonConfig) -> Self {
        Self {
            reload_time: 2.0,
            last_fire_time: -2.0, // spawn ready to fire.
            is_ready: true,
            is_firing: false,
            config,
        }
    }

    pub fn is_firing(&self) -> bool {
        self.is_firing
    }

    pub fn set_firing(&mut self, should_fire: bool) {
        self.is_firing = should_fire;
    }

    pub fn is_ready(&self) -> bool {
        self.is_ready
    }

    pub fn update(&mut self, current_time: f32) {
        self.is_ready = (current_time - self.last_fire_time) > self.reload_time
    }

    pub fn fired(&mut self, current_time: f32) {
        self.last_fire_time = current_time;
    }

    pub fn effect(&self) -> CannonFireEffect {
        self.config.fire_effect.clone()
    }
}
impl Component for Cannon {}

use crate::components::vehicle_interface::{Register, RegisterMap, VehicleModule};
pub struct CannonModule {
    entity: EntityId,
}

impl CannonModule {
    pub fn new(entity: EntityId) -> Self {
        CannonModule { entity }
    }
}

impl VehicleModule for CannonModule {
    fn get_registers(&self, world: &World, registers: &mut RegisterMap) {
        registers.clear();
        if let Some(cannon) = world.component::<Cannon>(self.entity) {
            registers.insert(0, Register::new_i32("firing", cannon.is_firing() as i32));
            registers.insert(1, Register::new_i32("ready", cannon.is_ready() as i32));
            registers.insert(2, Register::new_f32("reload_time", cannon.reload_time));
        }
    }

    fn set_component(&self, world: &mut World, registers: &RegisterMap) {
        if let Some(mut cannon) = world.component_mut::<Cannon>(self.entity) {
            let firing = registers
                .get(&0)
                .expect("register doesnt exist")
                .value_i32()
                .expect("wrong value type");
            cannon.set_firing(firing != 0);
        }
    }
}
