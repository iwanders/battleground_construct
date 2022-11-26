use battleground_construct::components;
use battleground_construct::systems;
use components::clock::{Clock, ClockSystem};
use engine::prelude::*;
use engine::Systems;

use components::acceleration::Acceleration;
use components::pose::Pose;
use components::velocity::Velocity;

#[test]
fn test_ball() {
    let mut world = World::new();

    let ball = world.add_entity();
    world.add_component(ball, Pose::new());
    world.add_component(ball, Velocity::new());
    world.add_component(ball, Acceleration::new());

    let clock_id = world.add_entity();
    world.add_component(clock_id, Clock::new());

    let mut systems = Systems::new();

    systems.add_system(Box::new(ClockSystem {}));
    systems.add_system(Box::new(
        systems::acceleration_velocity::AccelerationVelocity {},
    ));
    systems.add_system(Box::new(systems::velocity_pose::VelocityPose {}));
    systems.update(&mut world);

    {
        let pose = world
            .component::<Pose>(ball)
            .expect("Should have a pose for ball");
        assert_eq!(Pose::new().h, pose.h);
    }

    // Now, lets give the ball a velocity, 1m/s forwards, 1m/s upwards.
    // That gives us a 45 degree angle.
    {
        let mut vel = world
            .component_mut::<Velocity>(ball)
            .expect("Should have a velocity for ball");
        vel.v[0] = 1.0;
        vel.v[2] = 1.0;
    }
    // Add gravity.
    {
        let mut vel = world
            .component_mut::<Acceleration>(ball)
            .expect("Should have a acceleration for ball");
        vel.dv[2] = -9.81;
    }

    for i in 0..20 {
        let current_time = world
            .component::<Clock>(clock_id)
            .expect("Should have a clock")
            .elapsed_as_f32();
        systems.update(&mut world);
        let pose = world
            .component::<Pose>(ball)
            .expect("Should have a pose for ball");
        let vel = world
            .component::<Velocity>(ball)
            .expect("Should have a vel for ball");
        let accel = world
            .component::<Acceleration>(ball)
            .expect("Should have a accel for ball");
        let analytical_z = 1.0 * current_time - (9.81 / 2.0) * (current_time * current_time);
        println!("{i} {current_time:.5}    {x:.5} {y:.5} {z:.5},     {dx:.5}  {dy:.5} {dz:.5}  | {ddx}, {ddy}, {ddz:.5}  | {analytical_z:.5}  ",
            current_time = current_time,
            x = pose.h.w[0],y = pose.h.w[1],z = pose.h.w[2],
            dx = vel.v[0],dy = vel.v[1],dz = vel.v[2],
            ddx = accel.dv[0],ddy = accel.dv[1],ddz = accel.dv[2]);
        assert!((pose.h.w[2] - analytical_z).abs() < 0.01);
    }
}
