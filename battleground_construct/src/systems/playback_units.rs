use super::components;
use crate::units;
use crate::units::*;
use engine::prelude::*;

/// This system reconstructs units for the playback.
pub struct PlaybackUnits {}
impl System for PlaybackUnits {
    fn update(&mut self, world: &mut World) {
        for entity in world.component_entities::<units::tank::UnitTank>() {
            let needs_spawn = world
                .component::<components::recording::PlaybackUnitCreatedMarker>(entity)
                .is_none();
            let is_destroyed = world
                .component::<components::recording::PlaybackUnitDestroyedMarker>(entity)
                .is_some();
            let health_present = world
                .component::<components::health::Health>(entity)
                .is_some();

            if needs_spawn && health_present {
                let tank_unit = *world.component::<units::tank::UnitTank>(entity).unwrap();
                tank::add_tank_passive(world, &tank_unit);
                world.add_component(
                    tank_unit.unit_entity,
                    components::recording::PlaybackUnitCreatedMarker,
                );
            }
            if !health_present && !is_destroyed {
                let tank_unit = *world.component::<units::tank::UnitTank>(entity).unwrap();
                world.remove_entities(&tank_unit.children());
                world.add_component(
                    tank_unit.unit_entity,
                    components::recording::PlaybackUnitDestroyedMarker,
                );
            }
        }

        for entity in world.component_entities::<units::artillery::UnitArtillery>() {
            let needs_spawn = world
                .component::<components::recording::PlaybackUnitCreatedMarker>(entity)
                .is_none();
            let is_destroyed = world
                .component::<components::recording::PlaybackUnitDestroyedMarker>(entity)
                .is_some();
            let health_present = world
                .component::<components::health::Health>(entity)
                .is_some();

            if needs_spawn && health_present {
                let artillery_unit = *world
                    .component::<units::artillery::UnitArtillery>(entity)
                    .unwrap();
                artillery::add_artillery_passive(world, &artillery_unit);
                world.add_component(
                    artillery_unit.unit_entity,
                    components::recording::PlaybackUnitCreatedMarker,
                );
            }
            if !health_present && !is_destroyed {
                let artillery_unit = *world
                    .component::<units::artillery::UnitArtillery>(entity)
                    .unwrap();
                world.remove_entities(&artillery_unit.children());
                world.add_component(
                    artillery_unit.unit_entity,
                    components::recording::PlaybackUnitDestroyedMarker,
                );
            }
        }

        for entity in world.component_entities::<units::capturable_flag::UnitCapturableFlag>() {
            let needs_spawn = world
                .component::<components::recording::PlaybackUnitCreatedMarker>(entity)
                .is_none();

            if needs_spawn {
                let unit = *world
                    .component::<units::capturable_flag::UnitCapturableFlag>(entity)
                    .unwrap();
                units::capturable_flag::add_capturable_passives(world, &unit);
                world.add_component(entity, components::recording::PlaybackUnitCreatedMarker);
            }
        }
    }
}
