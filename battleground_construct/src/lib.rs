// https://rust-lang.github.io/api-guidelines/naming.html

pub mod components;
pub mod display;
pub mod systems;
use components::clock::{Clock, ClockSystem};
use engine::prelude::*;
use engine::Systems;
use crate::display::primitives::{Vec3, Mat4};

pub struct Construct {
    world: World,
    systems: Systems,
}

impl Construct {
    pub fn new() -> Self {
        let mut world = World::new();
        let clock_id = world.add_entity();
        world.add_component(&clock_id, Clock::new());


        for x in 0..1 {
            for y in 0..1 {

                let vehicle_id  = world.add_entity();
                let mut pose = components::pose::Pose::new();
                pose.h.w[0] =  (x as f32) * 5.0;
                pose.h.w[1] =  (y as f32) * 5.0;
                world.add_component(&vehicle_id, pose);
                world.add_component(&vehicle_id, components::velocity::Velocity::new());
                world.add_component(
                    &vehicle_id,
                    components::differential_drive_base::DifferentialDriveBase::new(),
                );
                world.add_component(&vehicle_id, display::tank_body::TankBody::new());

                let turret_offset = world.add_entity();
                let mut pose = components::pose::Pose::new();
                pose.h.w[2] =  0.85;
                world.add_component(&turret_offset, pose);
                world.add_component(&turret_offset, components::parent::Parent::new(vehicle_id.clone()));

                let turret_id = world.add_entity();
                let mut turret_revolute = components::revolute::Revolute::new_with_axis(Vec3::new(0.0, 0.0, 1.0));
                turret_revolute.set_velocity(0.1);

                world.add_component(&turret_id, turret_revolute);
                world.add_component(&turret_id, components::pose::Pose::new());
                world.add_component(&turret_id, components::parent::Parent::new(turret_offset.clone()));
                world.add_component(&turret_id, display::tank_turret::TankTurret::new());
            }
        }


        let mut systems = engine::Systems::new();
        systems.add_system(Box::new(ClockSystem {}));
        systems.add_system(Box::new(
            systems::kinematics_differential_drive::KinematicsDifferentialDrive {},
        ));
        systems.add_system(Box::new(systems::velocity_pose::VelocityPose{}));
        systems.add_system(Box::new(systems::revolute_pose::RevolutePose{}));

        Construct {
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
