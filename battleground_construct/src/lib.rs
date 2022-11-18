pub mod components;
pub mod display;
pub mod systems;
use components::clock::{Clock, ClockSystem};
use engine::prelude::*;
use engine::Systems;

pub struct Construct {
    vehicle_id: EntityId,
    world: World,
    systems: Systems,
}

impl Construct {
    pub fn new() -> Self {
        let mut world = World::new();
        let clock_id = world.add_entity();
        world.add_component(&clock_id, Clock::new());

        let vehicle_id = world.add_entity();
        world.add_component(&vehicle_id, components::pose::Pose::new());
        world.add_component(&vehicle_id, components::velocity::Velocity::new());
        world.add_component(
            &vehicle_id,
            components::differential_drive_base::DifferentialDriveBase::new(),
        );
        world.add_component(&vehicle_id, display::tank_body::TankBody::new());

        let mut systems = engine::Systems::new();
        systems.add_system(Box::new(ClockSystem {}));
        systems.add_system(Box::new(
            systems::kinematics_differential_drive::KinematicsDifferentialDrive {},
        ));
        systems.add_system(Box::new(systems::velocity_pose::VelocityPose {}));

        Construct {
            vehicle_id,
            world,
            systems,
        }
    }

    pub fn update(&mut self) {
        self.systems.update(&mut self.world);
    }

    pub fn world(&self) -> &World {
        &self.world
    }

    pub fn vehicle_id(&self) -> EntityId {
        self.vehicle_id.clone()
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
        let (_entity, clock) = construct
            .world()
            .component_iter_mut::<Clock>()
            .next()
            .expect("Should have one clock");
        assert_eq!(clock.elapsed_as_f32(), 0.03);
    }
}
