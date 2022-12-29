use super::components;
use engine::prelude::*;

pub struct UnitControllerErrorCheck {}
impl System for UnitControllerErrorCheck {
    fn update(&mut self, world: &mut World) {
        let errored_entities: Vec<EntityId> = world
            .component_iter::<components::unit_controller::UnitController>()
            .filter(|(_e, v)| v.error().is_some())
            .map(|(e, _v)| e)
            .collect();

        for entity in errored_entities {
            // Do stuff... for now lets just print the error
            {
                let controller = world
                    .component::<components::unit_controller::UnitController>(entity)
                    .unwrap();
                println!(
                    "Controller for {entity:?} failed with: {:?}",
                    controller.error().unwrap()
                );
            }
            // Finally, apply the destroy marker.
            world.add_component(entity, components::destroyed::Destroyed::new());
        }
    }
}
