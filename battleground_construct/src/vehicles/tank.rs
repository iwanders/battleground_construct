use crate::components;
use crate::display;
use crate::display::primitives::Vec3;
use components::pose::{Pose, PreTransform};
use engine::prelude::*;

#[derive(Default)]
pub struct TankSpawnConfig {
    pub x: f32,
    pub y: f32,
    pub left_wheel: f32,
    pub right_wheel: f32,
    pub turret_velocity: f32,
    pub barrel_velocity: f32,
    pub shooting: bool,
}

fn cannon_function(world: &mut World, muzzle_pose: &Pose, cannon_entity: &EntityId) {
    use crate::components::acceleration::Acceleration;
    use crate::components::point_projectile::PointProjectile;
    use crate::components::velocity::Velocity;
    use crate::display::particle_emitter::ParticleEmitter;
    use components::pose::{Pose, PreTransform};
    let muzzle_velocity = 20.0;
    // Get the pose of the cannon in the world coordinates. Then create the pose with the
    // Orientation in the global frame.
    let projectile_id = world.add_entity();
    world.add_component::<PointProjectile>(
        &projectile_id,
        PointProjectile::new(cannon_entity.clone()),
    );
    world.add_component::<Pose>(
        &projectile_id,
        Pose::from_mat4(cgmath::Matrix4::<f32>::from_translation(
            muzzle_pose.clone().w.truncate(),
        )),
    );

    // Calculate the velocity vector in the global frame.
    let mut muzzle_pose = muzzle_pose.transform().clone();
    // zero out the translation components.
    muzzle_pose.w[0] = 0.0;
    muzzle_pose.w[1] = 0.0;
    let v = muzzle_pose * cgmath::Vector4::<f32>::new(muzzle_velocity, 0.0, 0.0, 1.0);
    let projectile_velocity =
        Velocity::from_velocities(v.truncate(), cgmath::Vector3::<f32>::new(0.0, 0.0, 0.0));

    // And add the velocity to the projectile.
    world.add_component::<Velocity>(&projectile_id, projectile_velocity);
    // world.add_component(&projectile_id, crate::display::debug_box::DebugBox::from_size(0.2));
    world.add_component(
        &projectile_id,
        crate::display::tank_bullet::TankBullet::new(),
    );

    // Clearly not the place for this to be... but works for now.
    world.add_component(
        &projectile_id,
        crate::components::acceleration::Acceleration::gravity(),
    );

    world.add_component(
        &projectile_id,
        crate::display::particle_emitter::ParticleEmitter::bullet_trail(
            projectile_id,
            0.05,
            crate::display::Color::MAGENTA,
        ),
    );
}

pub fn spawn_tank(world: &mut World, config: TankSpawnConfig) {
    let mut tank_group_ids: Vec<EntityId> = vec![];
    use components::group::Group;
    use components::parent::Parent;
    use components::pose::{Pose, PreTransform};

    // Create the base tank, root element with the health.
    let vehicle_id = world.add_entity();
    tank_group_ids.push(vehicle_id.clone());
    let mut pose = Pose::new();

    pose.h.w[0] = config.x;
    pose.h.w[1] = config.y;
    world.add_component(&vehicle_id, pose);
    world.add_component(&vehicle_id, components::velocity::Velocity::new());
    let mut base = components::differential_drive_base::DifferentialDriveBase::new();
    base.set_velocities(config.left_wheel, config.right_wheel);
    world.add_component(&vehicle_id, base);
    world.add_component(&vehicle_id, display::tank_body::TankBody::new());
    world.add_component(&vehicle_id, display::tank_tracks::TankTracks::new());
    world.add_component(
        &vehicle_id,
        components::hit_sphere::HitSphere::with_radius(1.0),
    );
    // world.add_component(&vehicle_id, display::debug_sphere::DebugSphere::with_radius(1.0));
    world.add_component(&vehicle_id, components::health::Health::new());

    /*
     */

    // Add the turrent entity.
    let turret_id = world.add_entity();
    tank_group_ids.push(turret_id.clone());
    let mut turret_revolute =
        components::revolute::Revolute::new_with_axis(Vec3::new(0.0, 0.0, 1.0));
    turret_revolute.set_velocity(config.turret_velocity);

    world.add_component(&turret_id, turret_revolute);
    world.add_component(
        &turret_id,
        PreTransform::from_translation(Vec3::new(0.0, 0.0, 0.375 + 0.1 / 2.0)),
    );
    world.add_component(&turret_id, components::pose::Pose::new());
    world.add_component(&turret_id, Parent::new(vehicle_id.clone()));
    world.add_component(&turret_id, display::tank_turret::TankTurret::new());

    // Add the barrel entity
    let barrel_id = world.add_entity();
    tank_group_ids.push(barrel_id.clone());
    let mut barrel_revolute =
        components::revolute::Revolute::new_with_axis(Vec3::new(0.0, 1.0, 0.0));
    barrel_revolute.set_velocity(config.barrel_velocity);
    world.add_component(&barrel_id, barrel_revolute);
    world.add_component(
        &barrel_id,
        PreTransform::from_translation(Vec3::new(0.25, 0.0, 0.0)),
    );
    world.add_component(&barrel_id, components::pose::Pose::new());
    world.add_component(&barrel_id, Parent::new(turret_id.clone()));
    world.add_component(&barrel_id, display::tank_barrel::TankBarrel::new());

    // If the tank is shooting, at the nozzle and associated components
    let nozzle_id = if config.shooting {
        let nozzle_id = world.add_entity();
        tank_group_ids.push(nozzle_id.clone());
        world.add_component(&nozzle_id, Parent::new(barrel_id.clone()));
        world.add_component(
            &nozzle_id,
            components::damage_dealer::DamageDealer::new(0.1),
        );

        use crate::components::cannon::CannonFireEffect;
        let cannon_config = components::cannon::CannonConfig {
            reload_time: 2.0,
            fire_effect: std::rc::Rc::new(cannon_function),
        };

        world.add_component(&nozzle_id, components::cannon::Cannon::new(cannon_config));
        world.add_component(
            &nozzle_id,
            PreTransform::from_translation(Vec3::new(1.0, 0.0, 0.0)),
        );
        Some(nozzle_id)
    } else {
        None
    };

    let flag_id = world.add_entity();
    world.add_component(
        &flag_id,
        Pose::from_xyz(-0.8, -0.4, 0.3).rotated_angle_z(cgmath::Deg(180.0)),
    );
    world.add_component(
        &flag_id,
        display::flag::Flag::from_scale_color(0.5, display::Color::GREEN),
    );
    world.add_component(&flag_id, Parent::new(vehicle_id.clone()));
    tank_group_ids.push(flag_id.clone());

    // Finally, add the group to each of the components.
    world.add_component(&vehicle_id, Group::from(&tank_group_ids[..]));
    world.add_component(&turret_id, Group::from(&tank_group_ids[..]));
    world.add_component(&barrel_id, Group::from(&tank_group_ids[..]));
    if let Some(nozzle_id) = nozzle_id {
        world.add_component(&nozzle_id, Group::from(&tank_group_ids[..]));
    }
}
