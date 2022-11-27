use super::components::vehicle_controller::VehicleController;
use super::Clock;
use engine::prelude::*;

struct DummyInterface {}
impl battleground_vehicle_control::Interface for DummyInterface {
    fn registers(&self) -> usize {
        todo!()
    }
    fn get_u32(&self, _: usize) -> Result<u32, Box<(dyn std::error::Error + 'static)>> {
        todo!()
    }
    fn set_u32(&mut self, _: usize, _: u32) -> Result<u32, Box<(dyn std::error::Error + 'static)>> {
        todo!()
    }
}

pub struct VehicleControl {}
impl System for VehicleControl {
    fn update(&mut self, world: &mut World) {
        let time = world
            .component_iter_mut::<Clock>()
            .next()
            .expect("Should have one clock").1.elapsed_as_f32();

        // First, update the interface
        use std::collections::HashMap;
        use crate::components::vehicle_interface::{RegisterInterfaceContainer,RegisterInterface};
        let mut interfaces: Vec<(EntityId, RegisterInterfaceContainer)> = world.component_iter::<RegisterInterfaceContainer>().map(|(e, p)|{(e, p.clone())}).collect::<_>();
        let mut interface_map: HashMap<EntityId, RegisterInterfaceContainer> = interfaces.drain(..).collect::<_>();

        for (e, interface) in interface_map.iter_mut() {
            interface.get_mut().get_registers(world);
        }

        // Run all vehicle controls
        for (entity, mut controller) in world.component_iter_mut::<VehicleController>() {
            if let Some(controller) = controller.should_update(time) {
                if let Some(interface) = interface_map.get_mut(&entity) {
                    controller.update(&mut *interface.get_mut());
                }
            }
        }
        for (e, interface) in interface_map.iter_mut() {
            interface.get_mut().set_components(world);
        }

        /*
        */
    }
}
