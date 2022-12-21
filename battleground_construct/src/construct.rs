
use crate::components;
use engine::prelude::*;
use engine::Systems;

pub struct Construct {
    world: World,
    systems: Systems,
}

#[allow(clippy::new_without_default)]
impl Construct {
    pub fn new() -> Self {
        let world = World::new();
        let systems = engine::Systems::new();
        Construct { world, systems }
    }

    pub fn world_systems(&mut self) -> (&mut World, &mut Systems) {
        (&mut self.world, &mut self.systems)
    }

    pub fn update(&mut self) {
        self.systems.update(&mut self.world);
    }

    pub fn world(&self) -> &World {
        &self.world
    }

    pub fn entity_pose(&self, entity: EntityId) -> components::pose::Pose {
        components::pose::world_pose(&self.world, entity)
    }

    pub fn elapsed_as_f32(&self) -> f32 {
        let (_entity, clock) = self
            .world
            .component_iter_mut::<crate::components::clock::Clock>()
            .next()
            .expect("Should have one clock");
        clock.elapsed_as_f32()
    }
}
