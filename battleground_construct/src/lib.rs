// https://rust-lang.github.io/api-guidelines/naming.html

/*
    Todo:
        - Propagate velocities such that bullets get the correct initial velocity.
*/

pub mod components;
pub mod display;
pub mod systems;
pub mod util;
pub mod vehicles;
pub mod control;

// use battleground_vehicle_control;

use components::clock::{Clock, ClockSystem};
use engine::prelude::*;
use engine::Systems;
use vehicles::tank::{spawn_tank, TankSpawnConfig};

pub struct Construct {
    world: World,
    systems: Systems,
}

impl Construct {
    pub fn new() -> Self {
        let mut world = World::new();
        let clock_id = world.add_entity();
        world.add_component(clock_id, Clock::new());

        use components::function_pose::FunctionPose;
        use components::pose::Pose;
        use display::flag::Flag;
        use display::Color;

        let flag_id = world.add_entity();
        world.add_component(flag_id, components::pose::Pose::from_xyz(-1.0, -1.0, 0.0));
        world.add_component(flag_id, display::flag::Flag::new());

        let flag_id = world.add_entity();
        world.add_component(flag_id, components::pose::Pose::from_xyz(1.0, -1.0, 0.0));
        world.add_component(flag_id, Flag::from_scale_color(0.5, Color::GREEN));

        let particle_id = world.add_entity();
        world.add_component(particle_id, Pose::from_xyz(-1.0, -1.0, 0.0));
        world.add_component(particle_id, Flag::from_scale_color(0.5, Color::MAGENTA));
        world.add_component(
            particle_id,
            FunctionPose::new(|t| Pose::from_xyz(t.sin(), t.cos(), t.sin() + 1.0)),
        );
        world.add_component(
            particle_id,
            display::particle_emitter::ParticleEmitter::bullet_trail(
                particle_id,
                0.05,
                Color::WHITE,
            ),
        );
        /**/

        spawn_tank(
            &mut world,
            TankSpawnConfig {
                x: 0.0,
                y: 0.0,
                controller: Box::new(control::DummyVehicleControl{}),
            },
        );
        /**/

        for x in 1..2 {
            for y in 1..3 {
                spawn_tank(
                    &mut world,
                    TankSpawnConfig {
                        x: x as f32 * 5.0,
                        y: -y as f32 * 5.0 + 10.0,
                        controller: Box::new(control::DummyVehicleControl{}),
                    },
                );
            }
        }

        let mut systems = engine::Systems::new();
        systems.add_system(Box::new(ClockSystem {}));
        systems.add_system(Box::new(
            systems::kinematics_differential_drive::KinematicsDifferentialDrive {},
        ));
        systems.add_system(Box::new(
            systems::acceleration_velocity::AccelerationVelocity {},
        ));
        systems.add_system(Box::new(systems::velocity_pose::VelocityPose {}));
        systems.add_system(Box::new(systems::revolute_pose::RevolutePose {}));
        systems.add_system(Box::new(systems::cannon_trigger::CannonTrigger {}));
        systems.add_system(Box::new(systems::projectile_floor::ProjectileFloor {}));
        // systems.add_system(Box::new(systems::projectile_hit::ProjectileHit {}));
        // Must go after the hit calculation.
        systems.add_system(Box::new(systems::tank_hit_by::TankHitBy {}));
        // All handling of hits done with the projectiles still present.
        systems.add_system(Box::new(systems::health_tank_body::HealthTankBody {}));
        systems.add_system(Box::new(systems::display_tank_tracks::DisplayTankTracks {}));
        systems.add_system(Box::new(systems::function_pose::FunctionPose {}));
        systems.add_system(Box::new(systems::expiry_check::ExpiryCheck {}));
        systems.add_system(Box::new(systems::vehicle_control::VehicleControl {}));

        Construct { world, systems }
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

    pub fn elapsed_as_f64(&self) -> f64 {
        let (_entity, clock) = self
            .world
            .component_iter_mut::<crate::components::clock::Clock>()
            .next()
            .expect("Should have one clock");
        clock.elapsed_as_f32().into()
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
        assert_eq!(clock.elapsed_as_f32(), clock.step_as_f32() * 3.0);
    }
}
