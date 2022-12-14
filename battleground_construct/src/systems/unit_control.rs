use super::components::unit_controller::UnitController;
use super::Clock;
use engine::prelude::*;

pub struct UnitControl {}
impl System for UnitControl {
    fn update(&mut self, world: &mut World) {
        let time = world
            .component_iter_mut::<Clock>()
            .next()
            .expect("Should have one clock")
            .1
            .elapsed_as_f32();

        // First, collect the interfaces.
        use crate::components::unit_interface::RegisterInterfaceContainer;
        use std::collections::HashMap;

        // We only want to update the interfaces when the controller actually needs an update.
        // Otherwise we risk modifying components that should only be modified by the controller.
        let should_update = world
            .component_iter::<UnitController>()
            .filter(|(_e, c)| c.should_update(time))
            .map(|(e, _c)| e)
            .collect::<std::collections::HashSet<EntityId>>();

        // Create a map of entity -> interface
        let mut interfaces: Vec<(EntityId, RegisterInterfaceContainer)> = world
            .component_iter::<RegisterInterfaceContainer>()
            .filter(|(e, _p)| should_update.contains(e))
            .map(|(e, p)| (e, p.clone()))
            .collect::<_>();
        let mut interface_map: HashMap<EntityId, RegisterInterfaceContainer> =
            interfaces.drain(..).collect::<_>();

        // Then, the world is no longer borrowed and we can iterate over the interfaces, passing them the world.
        for (_e, interface) in interface_map.iter_mut() {
            interface.get_mut().get_registers(world);
        }

        // Run all vehicle controls, these ONLY work on interfaces.
        for (entity, mut controller) in world.component_iter_mut::<UnitController>() {
            if let Some(interface) = interface_map.get_mut(&entity) {
                let control = controller.vehicle_control();
                match control.update(&mut *interface.get_mut()) {
                    Ok(_) => {}
                    Err(v) => {
                        controller.set_error(v);
                    }
                }
                controller.set_updated(time);
            }
        }

        // Finally, we can again borrow the world and use the interfaces to write back to the world.
        for (_e, interface) in interface_map.iter_mut() {
            interface.get_mut().set_components(world);
        }
    }
}
