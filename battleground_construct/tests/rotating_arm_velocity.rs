use battleground_construct::components;
use battleground_construct::systems;
use cgmath::vec3;

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
    println!();
    println!();
    println!();
    println!();
    let mut world = World::new();

    let arm_origin_x = 5.0;
    let arm_length = 1.0;

    let arm_origin = world.add_entity();
    world.add_component(
        arm_origin,
        PreTransform::from_translation(Vec3::new(arm_origin_x, 0.0, 0.0)),
    );
    world.add_component(arm_origin, Pose::new());

    let arm_rotation = world.add_entity();
    world.add_component(arm_rotation, Parent::new(arm_origin.clone()));

    let mut arm_revolute = components::revolute::Revolute::new_with_axis(Vec3::new(0.0, 0.0, 1.0));
    let rotation_vel = 1.0;
    arm_revolute.set_velocity(1.0);
    world.add_component(arm_rotation, arm_revolute);
    world.add_component(arm_rotation, Pose::new());
    world.add_component(arm_rotation, Velocity::new());

    let arm_entity_pretransform = world.add_entity();
    world.add_component(arm_entity_pretransform, Parent::new(arm_rotation.clone()));
    world.add_component(arm_entity_pretransform, Pose::new());
    world.add_component(arm_entity_pretransform, Velocity::new());
    world.add_component(
        arm_entity_pretransform,
        PreTransform::from_translation(Vec3::new(arm_length, 0.0, 0.0)),
    );

    let arm_tip = world.add_entity();
    world.add_component(arm_tip, Pose::new());
    world.add_component(arm_tip, Parent::new(arm_entity_pretransform.clone()));

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

    // Get the arm rotation velocity, this should be equal to the rotation velocity.
    let arm_velocity = world.component::<Velocity>(arm_rotation).unwrap();
    println!("arm_velocity: {arm_velocity:?}");
    assert_eq!(arm_velocity.w.z, rotation_vel);

    // Get the arm tip position.
    let world_pose = components::pose::world_pose(&world, arm_tip);
    // This should be: x = 5 + 1 (minus a little bit, because rotation, in +y
    // y = a little bit, dt * rotation_vel * arm_length
    println!("world_pose: {world_pose:?}");
    assert!((world_pose.to_translation().x - (arm_origin_x + arm_length)).abs() < 0.001);
    assert!((world_pose.to_translation().y - 0.0).abs() < 0.001);

    // World velocity, should be, x = ~0, y = arm_length * rotation_vel, z =0;
    let world_velocity = components::velocity::world_velocity(&world, arm_tip);
    println!("world_velocity: {world_velocity:?}");
    assert!((world_velocity.v.x - (0.0)).abs() < 0.001);

    // Problem is in the last step, when we lift the calculation to the origin frame.
    // From the arm base to the origin is
    let origin_to_arm_base = Pose::from_se2(5.0, 0.0, 0.0);
    // Which is already wrong, but that's not the point here.
    let velocity_at_arm_base = Velocity::from_velocities(vec3(0.0, 0.0, 0.0), vec3(0.0, 0.0, 1.0));
    let vel_in_origin = origin_to_arm_base.to_adjoint() * velocity_at_arm_base.to_twist();
    println!("vel_in_origin: {vel_in_origin:?}");

    // So this here... poof.
    assert!((world_velocity.v.y - (arm_length * rotation_vel)).abs() < 0.001);

    println!();
    println!();
    println!();
    println!();

    // Now, retrieve the velocity of the arm.
    // let world_pose = components::pose::world_pose(&world, arm_tip);
    // println!("world_pose: {world_pose:?}");

    //
    // println!("world_velocity: {world_velocity:?}");
}
