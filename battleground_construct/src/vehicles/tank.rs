use crate::components;
use crate::display;
use crate::display::primitives::Vec3;
use components::pose::{Pose, PreTransform};
use engine::prelude::*;

pub const CLOCK_MODULE: u32 = 0x0100;
pub const BASE_MODULE: u32 = 0x1000;
pub const TURRET_MODULE: u32 = 0x1100;
pub const BARREL_MODULE: u32 = 0x1200;
pub const CANNON_MODULE: u32 = 0x1300;
pub const RADAR_ROTATION: u32 = 0x1500;
pub const RADAR_MODULE: u32 = 0x1600;
pub const LOCALIZATION_MODULE: u32 = 0x1700;

pub struct TankSpawnConfig {
    pub x: f32,
    pub y: f32,
    pub yaw: f32,
    pub controller: Box<dyn battleground_vehicle_control::VehicleControl>,
}

fn cannon_function(world: &mut World, muzzle_pose: &Pose, cannon_entity: EntityId) {
    use crate::components::point_projectile::PointProjectile;
    use crate::components::velocity::Velocity;

    let muzzle_velocity = 10.0;

    // Get the pose of the cannon in the world coordinates. Then create the pose with the
    // Orientation in the global frame.
    let projectile_id = world.add_entity();
    world.add_component::<PointProjectile>(
        projectile_id,
        PointProjectile::new(cannon_entity.clone()),
    );
    world.add_component::<Pose>(
        projectile_id,
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
    world.add_component::<Velocity>(projectile_id, projectile_velocity);
    // world.add_component(projectile_id, crate::display::debug_box::DebugBox::from_size(0.2));
    world.add_component(
        projectile_id,
        crate::display::tank_bullet::TankBullet::new(),
    );

    // Clearly not the place for this to be... but works for now.
    world.add_component(
        projectile_id,
        crate::components::acceleration::Acceleration::gravity(),
    );

    world.add_component(
        projectile_id,
        crate::display::particle_emitter::ParticleEmitter::bullet_trail(
            projectile_id,
            0.05,
            crate::display::Color::WHITE,
        ),
    );
}

pub fn spawn_tank(world: &mut World, config: TankSpawnConfig) -> EntityId {
    let mut tank_group_ids: Vec<EntityId> = vec![];
    use components::group::Group;
    use components::parent::Parent;

    // Create the base tank, root element with the health.
    let vehicle_id = world.add_entity();
    tank_group_ids.push(vehicle_id.clone());

    let register_interface = components::vehicle_interface::RegisterInterfaceContainer::new(
        components::vehicle_interface::RegisterInterface::new(),
    );

    world.add_component(vehicle_id, Pose::from_se2(config.x, config.y, config.yaw));
    world.add_component(vehicle_id, components::velocity::Velocity::new());
    let base = components::differential_drive_base::DifferentialDriveBase::new();
    // Register the base as a controllable.
    register_interface.get_mut().add_module(
        "clock",
        CLOCK_MODULE,
        components::clock::ClockReader::new(),
    );

    register_interface.get_mut().add_module(
        "base",
        BASE_MODULE,
        components::differential_drive_base::DifferentialDriveBaseModule::new(vehicle_id),
    );

    world.add_component(vehicle_id, base);
    world.add_component(vehicle_id, display::tank_tracks::TankTracks::new());

    let body_id = world.add_entity();
    world.add_component(
        body_id,
        PreTransform::from_translation(Vec3::new(0.0, 0.0, 0.25)),
    );
    let body = display::tank_body::TankBody::new();
    let hitbox = body.hitbox();
    world.add_component(body_id, body);
    world.add_component(body_id, hitbox);
    world.add_component(body_id, components::radar_reflector::RadarReflector::new());
    // world.add_component(body_id, components::hit_sphere::HitSphere::with_radius(1.0));
    world.add_component(body_id, Parent::new(vehicle_id.clone()));
    tank_group_ids.push(body_id.clone());

    let rc = components::vehicle_controller::VehicleControlStorage::new(config.controller);
    world.add_component(
        vehicle_id,
        components::vehicle_controller::VehicleController::new(rc),
    );
    // world.add_component(vehicle_id, display::debug_sphere::DebugSphere::with_radius(1.0));
    world.add_component(vehicle_id, components::health::Health::new());

    register_interface.get_mut().add_module(
        "localization",
        LOCALIZATION_MODULE,
        components::localization::LocalizationModule::new(body_id),
    );

    // Add the turrent entity.
    let turret_id = world.add_entity();

    // Register this revolute as a controllable.
    register_interface.get_mut().add_module(
        "turret",
        TURRET_MODULE,
        components::revolute::RevoluteModule::new(turret_id),
    );

    tank_group_ids.push(turret_id.clone());
    let turret_revolute = components::revolute::Revolute::new_with_axis(Vec3::new(0.0, 0.0, 1.0));

    world.add_component(turret_id, turret_revolute);
    world.add_component(
        turret_id,
        PreTransform::from_translation(Vec3::new(0.0, 0.0, 0.375 + 0.1 / 2.0)),
    );
    world.add_component(turret_id, components::pose::Pose::new());
    world.add_component(turret_id, components::velocity::Velocity::new());
    world.add_component(turret_id, Parent::new(vehicle_id.clone()));
    world.add_component(turret_id, display::tank_turret::TankTurret::new());

    // Add the barrel entity
    let barrel_id = world.add_entity();
    tank_group_ids.push(barrel_id.clone());
    let barrel_revolute = components::revolute::Revolute::new_with_axis(Vec3::new(0.0, 1.0, 0.0));
    register_interface.get_mut().add_module(
        "barrel",
        BARREL_MODULE,
        components::revolute::RevoluteModule::new(barrel_id),
    );

    world.add_component(barrel_id, barrel_revolute);
    world.add_component(
        barrel_id,
        PreTransform::from_translation(Vec3::new(0.25, 0.0, 0.0)),
    );
    world.add_component(barrel_id, components::pose::Pose::new());
    world.add_component(barrel_id, components::velocity::Velocity::new());
    world.add_component(barrel_id, Parent::new(turret_id.clone()));
    world.add_component(barrel_id, display::tank_barrel::TankBarrel::new());
    // world.add_component(barrel_id, display::debug_lines::DebugLines::straight(10.0, 0.1, display::primitives::Color::BLUE));

    // If the tank is shooting, add the nozzle and associated components
    let nozzle_id = world.add_entity();
    tank_group_ids.push(nozzle_id.clone());
    world.add_component(nozzle_id, Parent::new(barrel_id.clone()));
    world.add_component(nozzle_id, components::damage_dealer::DamageDealer::new(0.3));

    let cannon_config = components::cannon::CannonConfig {
        reload_time: 1.0,
        fire_effect: std::rc::Rc::new(cannon_function),
    };

    world.add_component(nozzle_id, components::cannon::Cannon::new(cannon_config));
    world.add_component(
        nozzle_id,
        PreTransform::from_translation(Vec3::new(1.0, 0.0, 0.0)),
    );

    register_interface.get_mut().add_module(
        "cannon",
        CANNON_MODULE,
        components::cannon::CannonModule::new(nozzle_id),
    );

    let radar_joint = world.add_entity();
    tank_group_ids.push(radar_joint.clone());
    let mut radar_revolute =
        components::revolute::Revolute::new_with_axis(Vec3::new(0.0, 0.0, 1.0));
    radar_revolute.set_velocity_bounds(-std::f32::consts::PI * 2.0, std::f32::consts::PI * 2.0);
    radar_revolute.set_velocity(-6.28);
    register_interface.get_mut().add_module(
        "radar_rotation",
        RADAR_ROTATION,
        components::revolute::RevoluteModule::new(radar_joint),
    );
    register_interface.get_mut().add_module(
        "radar",
        RADAR_MODULE,
        components::radar::RadarModule::new(radar_joint),
    );

    world.add_component(radar_joint, radar_revolute);
    // world.add_component(radar_joint, display::debug_lines::DebugLines::straight(10.0, 0.01, display::primitives::Color::RED));
    world.add_component(
        radar_joint,
        PreTransform::from_translation(Vec3::new(0.0, 0.0, 0.07)),
    );

    world.add_component(radar_joint, components::pose::Pose::new());
    world.add_component(radar_joint, components::velocity::Velocity::new());
    world.add_component(radar_joint, Parent::new(turret_id.clone()));
    // world.add_component(radar_joint, display::debug_box::DebugBox::new(0.1, 0.1, 0.1));
    world.add_component(radar_joint, display::radar_model::RadarModel::new());
    world.add_component(
        radar_joint,
        components::radar::Radar::new_with_config(components::radar::RadarConfig {
            range_max: 30.0,
            detection_angle_yaw: 1.0f32.to_radians(),
            detection_angle_pitch: 180f32.to_radians(),
            signal_strength: 1.0,
        }),
    );

    // Finally, add the register interface.
    world.add_component(vehicle_id, register_interface);

    let flag_id = world.add_entity();
    world.add_component(
        flag_id,
        Pose::from_xyz(-0.8, -0.4, 0.3).rotated_angle_z(cgmath::Deg(180.0)),
    );
    world.add_component(
        flag_id,
        display::flag::Flag::from_scale_color(0.5, display::Color::GREEN),
    );
    world.add_component(flag_id, Parent::new(vehicle_id.clone()));
    tank_group_ids.push(flag_id.clone());

    // Finally, add the group to each of the components.
    world.add_component(vehicle_id, Group::from(&tank_group_ids[..]));
    world.add_component(turret_id, Group::from(&tank_group_ids[..]));
    world.add_component(barrel_id, Group::from(&tank_group_ids[..]));
    world.add_component(nozzle_id, Group::from(&tank_group_ids[..]));
    world.add_component(body_id, Group::from(&tank_group_ids[..]));
    world.add_component(radar_joint, Group::from(&tank_group_ids[..]));
    vehicle_id
}
