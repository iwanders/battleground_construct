use super::components::vehicle_controller::VehicleController;
use super::Clock;
use engine::prelude::*;

pub struct VehicleControl {}
impl System for VehicleControl {
    fn update(&mut self, world: &mut World) {
        let time = world
            .component_iter_mut::<Clock>()
            .next()
            .expect("Should have one clock")
            .1
            .elapsed_as_f32();

        // First, collect the interfaces.
        use crate::components::vehicle_interface::RegisterInterfaceContainer;
        use std::collections::HashMap;

        // Create a map of entity -> interface
        let mut interfaces: Vec<(EntityId, RegisterInterfaceContainer)> = world
            .component_iter::<RegisterInterfaceContainer>()
            .map(|(e, p)| (e, p.clone()))
            .collect::<_>();
        let mut interface_map: HashMap<EntityId, RegisterInterfaceContainer> =
            interfaces.drain(..).collect::<_>();

        // Then, the world is no longer borrowed and we can iterate over the interfaces, passing them the world.
        for (_e, interface) in interface_map.iter_mut() {
            interface.get_mut().get_registers(world);
        }

        // Run all vehicle controls, these ONLY work on interfaces.
        for (entity, mut controller) in world.component_iter_mut::<VehicleController>() {
            if let Some(controller) = controller.should_update(time) {
                if let Some(interface) = interface_map.get_mut(&entity) {
                    controller.update(&mut *interface.get_mut());
                }
            }
        }

        // Finally, we can again borrow the world and use the interfaces to write back to the world.
        for (_e, interface) in interface_map.iter_mut() {
            interface.get_mut().set_components(world);
        }
    }
}
