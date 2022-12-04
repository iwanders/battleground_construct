use battleground_construct::components;
use battleground_construct::systems;
use components::clock::{Clock, ClockSystem};
use engine::prelude::*;
use engine::Systems;

use battleground_construct::display::primitives::Vec3;
use battleground_construct::util::cgmath::prelude::*;
use components::parent::Parent;
use components::pose::{Pose, PreTransform};
use components::velocity::Velocity;

#[test]
fn revolute_to_velocity() {
    let dt = 0.01;
    let mut revolute = components::revolute::Revolute::new_with_axis(Vec3::new(0.0, 0.0, 1.0));
    revolute.set_velocity(std::f32::consts::PI / 2.0);
    let pose_old = revolute.to_pose();
    revolute.integrate(dt);
    let pose_new = revolute.to_pose();

    let pose_new_direct = pose_new;
    println!("pose_old: {pose_old:?}");
    println!("pose_new: {pose_new:?}");

    // Roundtrip this.
    let vel_twist = revolute.to_twist();
    println!("vel_twist: {vel_twist:?}");
    // let pose_old = Pose::new();
    // let dh = vel * dt;
    let velocity: Velocity = vel_twist.into();
    // println!("dh: {dh:?}");
    let pose_new = velocity.integrate_pose(&pose_old, dt);
    let pose_new_through_twist = pose_new;
    assert_eq!(
        pose_new_through_twist.transform(),
        pose_new_direct.transform()
    );

    println!("pose_old: {pose_old:?}");
    println!("pose_new: {pose_new:?}");

    // Now, lets rotate with an arm.
    let arm = Pose::from_se2(1.0, 0.0, 0.0);
    println!("arm: {arm:?}");
    let arm_end_vel = arm.to_adjoint() * vel_twist;
    println!("arm_adjoint: {:?}", arm.to_adjoint());
    // rotating counter clockwise about z, looked from above, which is negative y.
    assert_eq!(arm_end_vel.v.y, -1.0);
    println!("arm_end_vel: {arm_end_vel:?}");
}

#[test]
fn test_rotating_arm() {
    let mut world = World::new();

    let arm_origin = world.add_entity();
    world.add_component(
        arm_origin,
        PreTransform::from_translation(Vec3::new(5.0, 0.0, 0.0)),
    );
    world.add_component(arm_origin, Pose::new());

    let arm_rotation = world.add_entity();
    // world.add_component(arm_rotation, Parent::new(arm_origin.clone()));

    let mut arm_revolute = components::revolute::Revolute::new_with_axis(Vec3::new(0.0, 0.0, 1.0));
    arm_revolute.set_velocity(1.0);
    world.add_component(arm_rotation, arm_revolute);
    world.add_component(arm_rotation, Pose::new());
    world.add_component(arm_rotation, Velocity::new());

    let arm_tip = world.add_entity();
    world.add_component(arm_tip, Parent::new(arm_rotation.clone()));
    world.add_component(arm_tip, Pose::new());
    world.add_component(arm_tip, Velocity::new());
    world.add_component(
        arm_tip,
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
    let world_pose = components::pose::world_pose(&world, arm_tip);
    println!("world_pose: {world_pose:?}");

    let world_velocity = components::velocity::world_velocity(&world, arm_tip);
    println!("world_velocity: {world_velocity:?}");
}
