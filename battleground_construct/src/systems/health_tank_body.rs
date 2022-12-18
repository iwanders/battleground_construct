use super::components::health::Health;
use crate::display::primitives::Color;
use crate::display::tank_body::TankBody;
use engine::prelude::*;

pub struct HealthTankBody {}
impl System for HealthTankBody {
    fn update(&mut self, world: &mut World) {
        for (entity, health) in world.component_iter::<Health>() {
            // try to see if we can find a velocity for this entity.
            if let Some(mut tank) = world.component_mut::<TankBody>(entity) {
                // Yes, so now integrate it.

                if health.is_destroyed() {
                    tank.set_color(Color {
                        r: 25,
                        g: 25,
                        b: 25,
                        a: 255,
                    });
                } else {
                    tank.set_color(Color {
                        r: (255.0 * (1.0 - health.health())) as u8,
                        g: (255.0 * (health.health())) as u8,
                        b: 0,
                        a: 255,
                    });
                }
            }
        }
    }
}
