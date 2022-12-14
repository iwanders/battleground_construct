use battleground_construct::components;
use battleground_construct::systems;
use cgmath::vec3;

use components::clock::{Clock, ClockSystem};
use engine::prelude::*;
use engine::Systems;

use battleground_construct::display::primitives::Vec3;
use battleground_construct::util::cgmath::prelude::*;
use components::parent::Parent;
use components::pose::{Pose, PreTransform, world_pose};
use components::velocity::Velocity;



#[test]
fn test_tank_body_velocities() {
    let mut world = World::new();

    let x = 4.9;
    let y = -5.7;
    let z = 1.0;

    // Create the base.
    let base_id = world.add_entity();

    world.add_component(base_id, Pose::from_translation(Vec3::new(x, y, z)));
    let mut vel = components::velocity::Velocity::new();
    world.add_component(base_id, vel);
    vel.v.x = 1.1;
    vel.w.z = 2.2;
    // world.add_component(vehicle_id, base);


    // Add the turrent entity.
    let turret_id = world.add_entity();

    let mut turret_revolute = components::revolute::Revolute::new_with_axis(Vec3::new(0.0, 0.0, 1.0));
    // turret_revolute.set_velocity_bounds(-std::f32::consts::PI * 2.0, std::f32::consts::PI * 2.0);
    turret_revolute.set_velocity(3.3);

    world.add_component(turret_id, turret_revolute);
    world.add_component(
        turret_id,
        PreTransform::from_translation(Vec3::new(0.0, 0.0, 1.0)),
    );
    world.add_component(turret_id, components::pose::Pose::new());
    let turret_vel = components::velocity::Velocity::new();
    world.add_component(turret_id, turret_vel);
    world.add_component(turret_id, Parent::new(base_id.clone()));
    // should update velocity.
    

    // Add the barrel linear offset, and joint.
    let barrel_id = world.add_entity();
    let mut barrel_revolute = components::revolute::Revolute::new_with_axis(Vec3::new(0.0, 1.0, 0.0));
    barrel_revolute.set_velocity(4.4);

    world.add_component(barrel_id, barrel_revolute);
    world.add_component(
        barrel_id,
        PreTransform::from_translation(Vec3::new(0.5, 0.0, 0.0)),
    );
    world.add_component(barrel_id, components::pose::Pose::new());
    world.add_component(barrel_id, components::velocity::Velocity::new());
    world.add_component(barrel_id, Parent::new(turret_id.clone()));
    // should update velocity.

    // Then, the arm from the joint to the center of the barrel.
    let barrel_cog_id = world.add_entity();
    world.add_component(barrel_cog_id, Pose::from_translation(Vec3::new(1.0, 0.0, 0.0)));
    world.add_component(barrel_cog_id, Parent::new(barrel_id.clone()));


    // Finally, a meter further from the center of the barrel, add the nozzle.
    let nozzle_id = world.add_entity();
    world.add_component(nozzle_id, Parent::new(barrel_cog_id.clone()));
    world.add_component(
        nozzle_id,
        PreTransform::from_translation(Vec3::new(1.0, 0.0, 0.0)),
    );

    let h_body_to_global = world_pose(&world, base_id);
    println!("h_body_to_global: {h_body_to_global:?}");

    let h_turret_to_global = world_pose(&world, turret_id);
    println!("h_turret_to_global: {h_turret_to_global:?}");

    let h_barrel_to_global = world_pose(&world, barrel_cog_id);
    println!("h_barrel_to_global: {h_barrel_to_global:?}");

    let h_nozzle_to_global = world_pose(&world, nozzle_id);
    println!("h_nozzle_to_global: {h_nozzle_to_global:?}");
}
