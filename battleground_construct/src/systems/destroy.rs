use super::components;

use engine::prelude::*;

pub struct Destroy {}
impl System for Destroy {
    fn update(&mut self, world: &mut World) {
        let destroyed = world.component_entities::<components::destroyed::Destroyed>();
        let destroyed_entity_and_root: Vec<(EntityId, EntityId)> = world
            .component_iter::<components::group::Group>()
            .filter(|(entity, _group)| destroyed.contains(entity))
            .map(|(entity, group)| (entity, group.entities()[0]))
            .collect::<_>();

        // We can now use the hit history

        // Right now, lets just nuke everything.
        let mut all_to_be_removed = vec![];

        for (_orig_entity, root_entity) in destroyed_entity_and_root.iter() {
            let mut elements_here = vec![];
            {
                let g = world
                    .component::<components::group::Group>(*root_entity)
                    .unwrap();
                elements_here.append(&mut g.entities().iter().copied().collect::<_>());
            }

            // Create the destruction effect.
            let thingy = world.add_entity();
            let mut destructor = crate::display::deconstructor::Deconstructor::new(thingy);

            // Now, we use the hit history to collect where the impacts were.
            if let Some(ref hit_history) =
                world.component::<components::hit_by::HitByHistory>(*root_entity)
            {
                for v in hit_history.hits() {
                    destructor.add_impact(v.impact().position(), 0.2);
                }
            }

            for e in elements_here.iter() {
                destructor.add_element::<crate::display::tank_body::TankBody>(*e, world);
                destructor.add_element::<crate::display::tank_turret::TankTurret>(*e, world);
                destructor.add_element::<crate::display::tank_barrel::TankBarrel>(*e, world);
                destructor.add_element::<crate::display::tank_tracks::TankTracks>(*e, world);
                destructor.add_element::<crate::display::radar_model::RadarModel>(*e, world);
            }
            world.add_component(thingy, destructor);
            world.add_component(thingy, crate::components::expiry::Expiry::lifetime(50.0));

            all_to_be_removed.append(&mut elements_here);
        }

        // Now, remove all these entities, later we will keep the tank id and statistics entity around.
        world.remove_entities(&all_to_be_removed);
    }
}
