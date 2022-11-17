pub mod components;
use components::clock::{Clock, ClockSystem};
use engine::prelude::*;
use engine::{Systems};


struct Construct {
    clock_id: EntityId,
    world: World,
    systems: Systems,
}

impl Construct {
    pub fn new() -> Self {
        let mut world = World::new();
        let clock_id = world.add_entity();
        world.add_component(&clock_id, Clock::new());
        let mut systems = engine::Systems::new();
        systems.add_system(Box::new(ClockSystem{}));
        Construct{
            clock_id,
            world,
            systems,
        }
    }

    pub fn update(&mut self) {
        self.systems.update(&mut self.world);
    }

    pub fn world(&mut self) -> &mut World{
        &mut self.world
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_things() {
        let mut construct = Construct::new();
        construct.update();
        construct.update();
        construct.update();
        let (_entity, mut clock) = construct.world()
            .component_iter_mut::<Clock>()
            .next()
            .expect("Should have one clock");
        assert_eq!(clock.elapsed_as_f32(), 0.03);
    }
}
