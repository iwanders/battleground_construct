use super::components;
use crate::units;
use crate::units::*;
use engine::prelude::*;

pub struct PlaybackUnits {}
impl System for PlaybackUnits {
    fn update(&mut self, world: &mut World) {
        let mut tanks = vec![];
        for (entity, tank_unit) in world.component_iter::<units::tank::UnitTank>() {
            let needs_spawn = !world
                .component::<components::recorder::PlaybackUnitCreatedMarker>(entity)
                .is_some();
            let is_destroyed = world
                .component::<components::recorder::PlaybackUnitDestroyedMarker>(entity)
                .is_some();
            let health_present = world
                .component::<components::health::Health>(entity)
                .is_some();
            tanks.push((tank_unit.clone(), needs_spawn, health_present, is_destroyed));
        }

        for (tank_unit, needs_spawn, health_present, is_destroyed) in tanks {
            if needs_spawn && health_present {
                tank::add_tank_passive(world, &tank_unit);
                world.add_component(
                    tank_unit.unit_entity,
                    components::recorder::PlaybackUnitCreatedMarker,
                );
            }
            if !health_present && !is_destroyed {
                world.remove_entities(&tank_unit.children());
                world.add_component(
                    tank_unit.unit_entity,
                    components::recorder::PlaybackUnitDestroyedMarker,
                );
                println!("Removing entities");
            }
        }
    }
}
