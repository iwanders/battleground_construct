use battleground_construct::components;

use cgmath::vec3;

use engine::prelude::*;

use battleground_construct::components::revolute::Revolute;
use battleground_construct::display::primitives::Vec3;
use battleground_construct::util::cgmath::prelude::*;
use cgmath::SquareMatrix;
use components::parent::Parent;
use components::pose::{world_pose, Pose, PreTransform};
use components::velocity::{world_velocity, Velocity};

#[test]
fn test_tank_body_velocities() {
    let mut world = World::new();

    let x = 4.9;
    let y = -5.7;
    let z = 1.0;

    // Create the base.
    let base_id = world.add_entity();
    println!("base_id: {base_id:?}");

    world.add_component(base_id, Pose::from_translation(Vec3::new(x, y, z)));
    let linear_velocity = 1.1;
    let angular_velocity = 2.2;
    let vel = Velocity::from_se2(linear_velocity, 0.0, angular_velocity);
    world.add_component(base_id, vel);

    // Add the turrent entity.
    let turret_id = world.add_entity();
    println!("turret_id: {turret_id:?}");

    let mut turret_revolute =
        components::revolute::Revolute::new_with_axis(Vec3::new(0.0, 0.0, 1.0));
    // turret_revolute.set_velocity_cmd_bounds(-std::f32::consts::PI * 2.0, std::f32::consts::PI * 2.0);
    turret_revolute.set_velocity_bounds(-100000.0, 100000.0);
    turret_revolute.set_velocity_cmd(3.3);
    turret_revolute.set_velocity_to_cmd();
    let turret_vel: components::velocity::Velocity = turret_revolute.to_twist().into();

    world.add_component(turret_id, turret_revolute);
    world.add_component(
        turret_id,
        PreTransform::from_translation(Vec3::new(0.0, 0.0, 1.0)),
    );
    world.add_component(turret_id, components::pose::Pose::new());
    world.add_component(turret_id, turret_vel);
    world.add_component(turret_id, Parent::new(base_id));

    // Add the barrel linear offset, and joint.
    let barrel_id = world.add_entity();
    println!("barrel_id: {barrel_id:?}");
    let mut barrel_revolute =
        components::revolute::Revolute::new_with_axis(Vec3::new(0.0, 1.0, 0.0));
    barrel_revolute.set_velocity_bounds(-100000.0, 100000.0);
    barrel_revolute.set_velocity_cmd(4.4);
    barrel_revolute.set_velocity_to_cmd();
    let barrel_vel: components::velocity::Velocity = barrel_revolute.to_twist().into();
    // let barrel_vel: components::velocity::Velocity = turret_revolute.to_twist().into();

    world.add_component(barrel_id, barrel_revolute);
    world.add_component(
        barrel_id,
        PreTransform::from_translation(Vec3::new(0.5, 0.0, 0.0)),
    );
    world.add_component(barrel_id, components::pose::Pose::new());
    world.add_component(barrel_id, barrel_vel);
    world.add_component(barrel_id, Parent::new(turret_id));

    // Then, the arm from the joint to the center of the barrel.
    let barrel_cog_id = world.add_entity();
    world.add_component(
        barrel_cog_id,
        PreTransform::from_translation(Vec3::new(1.0, 0.0, 0.0)),
    );
    world.add_component(barrel_cog_id, Parent::new(barrel_id));

    // Finally, a meter further from the center of the barrel, add the nozzle.
    let nozzle_id = world.add_entity();
    println!("nozzle_id: {nozzle_id:?}");
    world.add_component(nozzle_id, Parent::new(barrel_cog_id));
    world.add_component(
        nozzle_id,
        PreTransform::from_translation(Vec3::new(1.0, 0.0, 0.0)),
    );

    // Shell at the exit, with rotational velocity of 5.5 radians
    let shell_id = world.add_entity();
    println!("shell_id: {shell_id:?}");
    world.add_component(shell_id, Parent::new(nozzle_id));
    let mut shell_revolute =
        components::revolute::Revolute::new_with_axis(Vec3::new(1.0, 0.0, 0.0));

    shell_revolute.set_velocity_bounds(-100000.0, 100000.0);
    shell_revolute.set_velocity_cmd(5.5);
    shell_revolute.set_velocity_to_cmd();
    let shell_vel: components::velocity::Velocity = shell_revolute.to_twist().into();

    world.add_component(shell_id, shell_revolute);
    world.add_component(shell_id, shell_vel);

    // Print all H matrices, and check them again known good values.
    let h_body_to_global = world_pose(&world, base_id);
    println!("h_body_to_global: {h_body_to_global:?}");
    assert!(h_body_to_global.to_rotation().is_identity());
    assert_eq!(h_body_to_global.to_translation(), vec3(4.9, -5.7, 1.0));

    let h_turret_to_global = world_pose(&world, turret_id);
    println!("h_turret_to_global: {h_turret_to_global:?}");
    assert!(h_turret_to_global.to_rotation().is_identity());
    assert_eq!(h_turret_to_global.to_translation(), vec3(4.9, -5.7, 2.0));

    let h_barrel_to_global = world_pose(&world, barrel_cog_id);
    println!("h_barrel_to_global: {h_barrel_to_global:?}");
    assert!(h_barrel_to_global.to_rotation().is_identity());
    assert_eq!(h_barrel_to_global.to_translation(), vec3(6.4, -5.7, 2.0));

    let h_nozzle_to_global = world_pose(&world, nozzle_id);
    println!("h_nozzle_to_global: {h_nozzle_to_global:?}");
    assert!(h_nozzle_to_global.to_rotation().is_identity());
    assert_eq!(h_nozzle_to_global.to_translation(), vec3(7.4, -5.7, 2.0));

    let vel_body_in_global = world_velocity(&world, base_id);
    println!("vel_body_in_global: {vel_body_in_global:?}");
    assert_eq!(vel_body_in_global.w, vec3(0.0, 0.0, 2.2));
    assert_eq!(vel_body_in_global.v, vec3(1.1, 0.0, 0.0));

    let vel_turret_in_global = world_velocity(&world, turret_id);
    println!("vel_turret_in_global: {vel_turret_in_global:?}");
    assert_eq!(vel_turret_in_global.w, vec3(0.0, 0.0, 5.5));
    assert_eq!(vel_turret_in_global.v, vec3(1.1, 0.0, 0.0));

    let vel_barrel_cog_in_global = world_velocity(&world, barrel_cog_id);
    println!("vel_barrel_cog_in_global: {vel_barrel_cog_in_global:?}");
    assert_eq!(vel_barrel_cog_in_global.w, vec3(0.0, 4.4, 5.5));
    assert_eq!(vel_barrel_cog_in_global.v, vec3(1.1, 8.25, -4.4));

    let vel_shell_in_global = world_velocity(&world, shell_id);
    println!("vel_shell_in_global: {vel_shell_in_global:?}");
    assert_eq!(vel_shell_in_global.w, vec3(5.5, 4.4, 5.5));
    assert_eq!(vel_shell_in_global.v, vec3(1.1, 13.75, -8.8));

    // Now, lets change the turret and barrel orientations.
    {
        let new_revolute_turret = {
            let mut v = world.component_mut::<Revolute>(turret_id);
            let rev = v.as_mut().unwrap();

            rev.set_position(std::f32::consts::PI / 4.0);
            **rev
        };
        {
            let mut p = world.component_mut::<Pose>(turret_id);
            let pose = p.as_mut().unwrap();
            **pose = new_revolute_turret.to_pose();
        }
        {
            println!("Pose: {:?}", world.component_mut::<Pose>(turret_id));
            // panic!();
        }
        let new_revolute_barrel = {
            let mut v = world.component_mut::<Revolute>(barrel_id);
            let rev = v.as_mut().unwrap();

            rev.set_position(-std::f32::consts::PI / 4.0);
            **rev
        };
        {
            let mut p = world.component_mut::<Pose>(barrel_id);
            if let Some(ref mut pose) = p {
                **pose = new_revolute_barrel.to_pose();
            }
        }
    }

    // Check again.
    use battleground_construct::display::primitives::Mat4;
    use cgmath::Rad;

    let h_body_to_global = world_pose(&world, base_id);
    println!("h_body_to_global: {h_body_to_global:?}");
    assert!(h_body_to_global.to_rotation().is_identity());
    assert_eq!(h_body_to_global.to_translation(), vec3(4.9, -5.7, 1.0));

    let h_turret_to_global = world_pose(&world, turret_id);
    println!("h_turret_to_global: {h_turret_to_global:?}");
    assert_eq!(
        h_turret_to_global.transform().to_rotation_h(),
        Mat4::from_angle_z(Rad(std::f32::consts::PI / 4.0))
    );
    assert_eq!(h_turret_to_global.to_translation(), vec3(4.9, -5.7, 2.0));

    let h_barrel_to_global = world_pose(&world, barrel_cog_id);
    println!("h_barrel_to_global: {h_barrel_to_global:?}");
    assert_eq!(
        h_barrel_to_global.to_translation(),
        vec3(5.7535534, -4.8464465, 2.7071068)
    );
    assert_eq!(
        h_barrel_to_global.to_rotation_h(),
        Mat4::from_angle_z(Rad(std::f32::consts::PI / 4.0))
            * Mat4::from_angle_y(Rad(-std::f32::consts::PI / 4.0))
    );

    let h_nozzle_to_global = world_pose(&world, nozzle_id);
    println!("h_nozzle_to_global: {h_nozzle_to_global:?}");
    assert_eq!(
        h_nozzle_to_global.to_rotation_h(),
        Mat4::from_angle_z(Rad(std::f32::consts::PI / 4.0))
            * Mat4::from_angle_y(Rad(-std::f32::consts::PI / 4.0))
    );
    assert_eq!(
        h_nozzle_to_global.to_translation(),
        vec3(6.2535534, -4.3464465, 3.4142137)
    );

    let vel_body_in_global = world_velocity(&world, base_id);
    println!("vel_body_in_global: {vel_body_in_global:?}");
    assert_eq!(vel_body_in_global.w, vec3(0.0, 0.0, 2.2));
    assert_eq!(vel_body_in_global.v, vec3(1.1, 0.0, 0.0));

    let vel_turret_in_global = world_velocity(&world, turret_id);
    println!("vel_turret_in_global: {vel_turret_in_global:?}");
    assert_eq!(vel_turret_in_global.w, vec3(0.0, 0.0, 5.5));
    assert_eq!(vel_turret_in_global.v, vec3(1.1, 0.0, 0.0));

    let vel_barrel_in_global = world_velocity(&world, barrel_id);
    println!("vel_barrel_in_global: {vel_barrel_in_global:?}");
    assert_eq!(vel_barrel_in_global.w, vec3(-3.11127, 3.11127, 5.5));
    assert_eq!(vel_barrel_in_global.v, vec3(-0.84454346, 1.9445434, 0.0));

    let vel_barrel_cog_in_global = world_velocity(&world, barrel_cog_id);
    println!("vel_barrel_cog_in_global: {vel_barrel_cog_in_global:?}");
    assert_eq!(vel_barrel_cog_in_global.w, vec3(-3.11127, 3.11127, 5.5));
    assert_eq!(
        vel_barrel_cog_in_global.v,
        vec3(-1.3945432, 6.894543, -3.111_27)
    );

    let vel_shell_in_global = world_velocity(&world, shell_id);
    println!("vel_shell_in_global: {vel_shell_in_global:?}");
    assert_eq!(
        vel_shell_in_global.v,
        vec3(-1.9445434, 11.844543, -6.222_54)
    );
    assert_eq!(vel_shell_in_global.w, vec3(-0.36127007, 5.861_27, 9.389088));
}
