use super::components::health::Health;
use crate::display::health_bar::HealthBar;
use engine::prelude::*;

pub struct HealthBarUpdate {}
impl System for HealthBarUpdate {
    fn update(&mut self, world: &mut World) {
        for (_entity, mut health_bar) in world.component_iter_mut::<HealthBar>() {
            if let Some(health) = world.component::<Health>(health_bar.health_entity()) {
                health_bar.set_health(health.health());
            }
        }
    }
}
