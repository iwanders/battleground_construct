use super::components::gun_battery::GunBattery;

use super::Clock;
use engine::prelude::*;

pub struct GunBatteryTrigger {}
impl System for GunBatteryTrigger {
    fn update(&mut self, world: &mut World) {
        let current = {
            let (_entity, clock) = world
                .component_iter_mut::<Clock>()
                .next()
                .expect("Should have one clock");
            clock.elapsed_as_f32()
        };

        for gun_entity in world.component_entities::<GunBattery>() {
            let mut fire_poses = vec![];
            {
                let mut gun_battery = world.component_mut::<GunBattery>(gun_entity).unwrap();
                gun_battery.update(current);
                while gun_battery.is_triggered() && gun_battery.is_ready() {
                    fire_poses.push(gun_battery.fired(current))
                }
            };

            let fire_effect = { world.component::<GunBattery>(gun_entity).unwrap().effect() };
            for pose in fire_poses {
                fire_effect(world, gun_entity, pose);
            }
        }
    }
}
