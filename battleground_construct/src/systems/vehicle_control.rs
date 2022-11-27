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
        let (_entity, clock) = world
            .component_iter_mut::<Clock>()
            .next()
            .expect("Should have one clock");
        let time = clock.elapsed_as_f32();

        // Run all vehicle controls
        for (entity, mut controller) in world.component_iter_mut::<VehicleController>() {
            if let Some(controller) = controller.should_update(time) {
                let mut dummy = DummyInterface {};
                controller.update(&mut dummy);
            }
        }
    }
}
