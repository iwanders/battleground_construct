// https://rust-lang.github.io/api-guidelines/naming.html

pub mod components;
pub mod display;
pub mod systems;
use components::clock::{Clock, ClockSystem};
use engine::prelude::*;
use engine::Systems;
use crate::display::primitives::{Vec3};

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

                let turret_id = world.add_entity();
                let mut turret_revolute = components::revolute::Revolute::new_with_axis(Vec3::new(0.0, 0.0, 1.0));
                turret_revolute.set_velocity(0.1);

                world.add_component(&turret_id, turret_revolute);
                world.add_component(&turret_id, components::pose::PreTransform::from_translation(Vec3::new(0.0, 0.0, 0.375 + 0.1 / 2.0)));
                world.add_component(&turret_id, components::pose::Pose::new());
                world.add_component(&turret_id, components::parent::Parent::new(vehicle_id.clone()));
                world.add_component(&turret_id, display::tank_turret::TankTurret::new());


                let barrel_id = world.add_entity();
                let mut barrel_revolute = components::revolute::Revolute::new_with_axis(Vec3::new(0.0, 1.0, 0.0));
                barrel_revolute.set_velocity(1.5);
                world.add_component(&barrel_id, barrel_revolute);
                world.add_component(&barrel_id, components::pose::PreTransform::from_translation(Vec3::new(0.25, 0.0, 0.0)));
                world.add_component(&barrel_id, components::pose::Pose::new());
                world.add_component(&barrel_id, components::parent::Parent::new(turret_id.clone()));
                world.add_component(&barrel_id, display::tank_barrel::TankBarrel::new());


                let nozzle_id = world.add_entity();
                world.add_component(&nozzle_id, components::parent::Parent::new(barrel_id.clone()));
                world.add_component(&nozzle_id, components::pose::PreTransform::from_translation(Vec3::new(1.0, 0.0, 0.0)));
                world.add_component(&nozzle_id, display::debug_box::DebugBox::from_size(0.2));
                
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

    pub fn entity_pose(&self, entity: &EntityId) -> components::pose::Pose {
        let mut current_id = entity.clone();
        let mut current_pose = components::pose::Pose::new();
        loop {
            let pose = self.world().component::<components::pose::Pose>(&current_id);
            if let Some(pose) = pose {
                    current_pose = *pose * current_pose;
            }
            let pre_pose = self.world().component::<components::pose::PreTransform>(&current_id);
            if let Some(pre_pose) = pre_pose {
                    current_pose = (pre_pose.transform() * current_pose.transform()).into();
            }
            if let Some(parent) = self
                .world()
                .component::<components::parent::Parent>(&current_id)
            {
                current_id = parent.parent().clone();
            } else {
                break;
            }
        }
        current_pose
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
