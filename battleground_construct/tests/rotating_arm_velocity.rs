use battleground_construct::components;
use battleground_construct::systems;
use components::clock::{Clock, ClockSystem};
use engine::prelude::*;
use engine::Systems;

use components::acceleration::Acceleration;
use components::pose::{Pose, PreTransform};
use components::velocity::Velocity;
use components::parent::Parent;
use battleground_construct::display::primitives::Vec3;


#[test]
fn revolute_to_velocity() {
    let dt = 0.01;
    let mut revolute = components::revolute::Revolute::new_with_axis(Vec3::new(0.0, 0.0, 1.0));
    revolute.set_velocity(std::f32::consts::PI / 2.0);
    let pose_old = revolute.to_pose();
    revolute.integrate(dt);
    let pose_new = revolute.to_pose();

    println!("pose_old: {pose_old:?}");
    println!("pose_new: {pose_new:?}");

    let vel = revolute.to_twist();
    println!("vel: {vel:?}");
    // let pose_old = Pose::new();
    // let dh = vel * dt;
    let velocity: Velocity = vel.into();
    // println!("dh: {dh:?}");
    let pose_new = velocity.integrate_pose(&pose_old, dt);
    println!("pose_old: {pose_old:?}");
    println!("pose_new: {pose_new:?}");

}


#[test]
fn test_rotating_arm() {
    
    let mut world = World::new();

    let arm_origin = world.add_entity();
    world.add_component(
        arm_origin,
        PreTransform::from_translation(Vec3::new(2.0, 0.0, 1.0)),
    );
    world.add_component(arm_origin, Pose::new());


    let arm_rotation = world.add_entity(); 
    world.add_component(arm_rotation, Parent::new(arm_origin.clone()));

    let mut arm_revolute = components::revolute::Revolute::new_with_axis(Vec3::new(0.0, 0.0, 1.0));
    arm_revolute.set_velocity(1.0);
    world.add_component(arm_rotation, arm_revolute);
    world.add_component(arm_rotation, Pose::new());



    let arm = world.add_entity();
    world.add_component(arm, Parent::new(arm_rotation.clone()));
    world.add_component(arm, Pose::new());
    world.add_component(
        arm,
        PreTransform::from_translation(Vec3::new(1.0, 0.0, 0.0)),
    );

    let clock_id = world.add_entity();
    world.add_component(clock_id, Clock::new());

    let mut systems = Systems::new();

    systems.add_system(Box::new(ClockSystem {}));
    systems.add_system(Box::new(
        systems::acceleration_velocity::AccelerationVelocity {},
    ));
    systems.add_system(Box::new(systems::revolute_velocity::RevoluteVelocity {}));
    // systems.add_system(Box::new(systems::revolute_pose::RevolutePose {}));
    systems.add_system(Box::new(systems::velocity_pose::VelocityPose {}));
    systems.update(&mut world);

    // Now, retrieve the velocity of the arm.
    let world_pose = components::pose::world_pose(&world, arm);
    println!("world_pose: {world_pose:?}");

    let world_velocity = components::velocity::world_velocity(&world, arm);
    println!("world_velocity: {world_velocity:?}");
}
