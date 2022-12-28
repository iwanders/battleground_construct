use crate::components;
use crate::display;
use crate::display::primitives::Vec3;
use components::group::Group;
use components::parent::Parent;
use components::pose::{Pose, PreTransform};
use engine::prelude::*;

use battleground_unit_control::units::tank::*;

pub struct TankSpawnConfig {
    pub x: f32,
    pub y: f32,
    pub yaw: f32,
    pub controller: Box<dyn battleground_unit_control::UnitControl>,
    pub team_member: Option<components::team_member::TeamMember>,
    pub radio_config: Option<super::common::RadioConfig>,
}

impl Default for TankSpawnConfig {
    fn default() -> Self {
        TankSpawnConfig {
            x: 0.0,
            y: 0.0,
            yaw: 0.0,
            controller: Box::new(unit_control_builtin::idle::Idle {}),
            team_member: None,
            radio_config: None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct UnitTank {
    pub unit_entity: EntityId,
    pub control_entity: EntityId,
    pub base_entity: EntityId,
    pub body_entity: EntityId,
    pub turret_entity: EntityId,
    pub radar_entity: EntityId,
    pub flag_entity: EntityId,
    pub barrel_entity: EntityId,
    pub nozzle_entity: EntityId,
}
impl Component for UnitTank {}

pub fn spawn_tank(world: &mut World, config: TankSpawnConfig) -> EntityId {
    /*
        Topology of the tank;

        Unit Entity:
            - Health
            - TeamMember
            - Eternal

        Control Entity:
            - UnitController

        Base Entity:
            - Diff Drive controller
            -> Body entity
                - RadarReflector
                - CaptureMarker
                - Radio's
            -> Flag entity
            -> Turret Entity
                - Revolute
                -> Barrel Entity
                    -> Nozzle Entity
                -> Radar entity

        The Unit and Control entities are 'free'.
        Base to Barrel forms a chain of Parent, all entities are part of the group.
    */
    let unit_entity = world.add_entity();
    let control_entity = world.add_entity();

    let base_entity = world.add_entity();
    let body_entity = world.add_entity();
    let turret_entity = world.add_entity();
    let radar_entity = world.add_entity();
    let flag_entity = world.add_entity();
    let barrel_entity = world.add_entity();
    let nozzle_entity = world.add_entity();

    let tank_group_entities: Vec<EntityId> = vec![
        unit_entity,
        control_entity,
        base_entity,
        body_entity,
        turret_entity,
        radar_entity,
        flag_entity,
        barrel_entity,
        nozzle_entity,
    ];
    let unit_tank = UnitTank {
        unit_entity,
        control_entity,
        base_entity,
        body_entity,
        turret_entity,
        radar_entity,
        flag_entity,
        barrel_entity,
        nozzle_entity,
    };

    // Create the register interface, we'll add modules throughout this function.
    let register_interface = components::unit_interface::RegisterInterfaceContainer::new(
        components::unit_interface::RegisterInterface::new(),
    );

    // -----    Global modules
    register_interface.get_mut().add_module(
        "clock",
        MODULE_CLOCK,
        components::clock::ClockModule::new(),
    );
    register_interface.get_mut().add_module(
        "objectives",
        MODULE_OBJECTIVES,
        components::objectives_module::ObjectivesModule::new(),
    );

    // -----   Unit
    world.add_component(unit_entity, components::health::Health::new());
    world.add_component(unit_entity, components::eternal::Eternal::new());
    register_interface.get_mut().add_module(
        "team",
        MODULE_TEAM,
        components::team_module::TeamModule::new(unit_entity),
    );
    world.add_component(unit_entity, unit_tank);

    let unit_component = components::unit::Unit::new(
        components::id_generator::generate_id(world),
        battleground_unit_control::units::UnitType::Tank,
    );
    let unit_id = unit_component.id();
    world.add_component(unit_entity, unit_component);

    register_interface.get_mut().add_module(
        "unit",
        MODULE_UNIT,
        components::unit::UnitModuleComponent::new(unit_entity),
    );

    // -----   Base
    world.add_component(base_entity, Pose::from_se2(config.x, config.y, config.yaw));
    world.add_component(base_entity, components::velocity::Velocity::new());
    world.add_component(
        base_entity,
        components::differential_drive_base::DifferentialDriveBase::new(),
    );
    register_interface.get_mut().add_module(
        "diff_drive",
        MODULE_DIFF_DRIVE,
        components::differential_drive_base::DifferentialDriveBaseModule::new(base_entity),
    );
    world.add_component(base_entity, display::tank_tracks::TankTracks::new());

    // -----   Body
    world.add_component(
        body_entity,
        PreTransform::from_translation(Vec3::new(0.0, 0.0, 0.25)),
    );
    let body = display::tank_body::TankBody::new();
    let hitbox = body.hitbox();
    world.add_component(body_entity, body);
    world.add_component(body_entity, hitbox);
    world.add_component(
        body_entity,
        components::radar_reflector::RadarReflector::new(),
    );
    world.add_component(
        body_entity,
        components::capture_marker::CaptureMarker::new(),
    );
    world.add_component(body_entity, Parent::new(base_entity));

    // Lets place drawing and gps in the base as well.
    world.add_component(body_entity, display::draw_module::DrawComponent::new());
    register_interface.get_mut().add_module(
        "draw",
        MODULE_DRAW,
        display::draw_module::DrawModule::new(body_entity),
    );
    register_interface.get_mut().add_module(
        "localization",
        MODULE_GPS,
        components::gps::GpsModule::new(body_entity),
    );

    // Radios are also on the body, because the gps is also there.
    let transmitter_config = config
        .radio_config
        .map(|v| components::radio_transmitter::RadioTransmitterConfig {
            channel_min: v.channel_min,
            channel_max: v.channel_max,
            ..Default::default()
        })
        .unwrap_or_default();
    world.add_component(
        body_entity,
        components::radio_transmitter::RadioTransmitter::new_with_config(transmitter_config),
    );
    register_interface.get_mut().add_module(
        "radio_transmitter",
        MODULE_RADIO_TRANSMITTER,
        components::radio_transmitter::RadioTransmitterModule::new(body_entity),
    );

    let receiver_config = config
        .radio_config
        .map(|v| components::radio_receiver::RadioReceiverConfig {
            channel_min: v.channel_min,
            channel_max: v.channel_max,
            ..Default::default()
        })
        .unwrap_or_default();
    world.add_component(
        body_entity,
        components::radio_receiver::RadioReceiver::new_with_config(receiver_config),
    );
    register_interface.get_mut().add_module(
        "radio_receiver",
        MODULE_RADIO_RECEIVER,
        components::radio_receiver::RadioReceiverModule::new(body_entity),
    );

    // -----   Turret
    register_interface.get_mut().add_module(
        "turret",
        MODULE_REVOLUTE_TURRET,
        components::revolute::RevoluteModule::new(turret_entity),
    );

    let turret_revolute = components::revolute::Revolute::new_with_axis(Vec3::new(0.0, 0.0, 1.0));
    world.add_component(turret_entity, turret_revolute);
    world.add_component(
        turret_entity,
        PreTransform::from_translation(Vec3::new(0.0, 0.0, 0.375 + 0.1 / 2.0)),
    );
    world.add_component(turret_entity, components::pose::Pose::new());
    world.add_component(turret_entity, components::velocity::Velocity::new());
    world.add_component(turret_entity, Parent::new(base_entity));
    world.add_component(turret_entity, display::tank_turret::TankTurret::new());

    // -----   Barrel
    let barrel_revolute = components::revolute::Revolute::new_with_axis(Vec3::new(0.0, 1.0, 0.0));
    register_interface.get_mut().add_module(
        "barrel",
        MODULE_REVOLUTE_BARREL,
        components::revolute::RevoluteModule::new(barrel_entity),
    );

    world.add_component(barrel_entity, barrel_revolute);
    world.add_component(
        barrel_entity,
        PreTransform::from_translation(Vec3::new(0.25, 0.0, 0.0)),
    );
    world.add_component(barrel_entity, components::pose::Pose::new());
    world.add_component(barrel_entity, components::velocity::Velocity::new());
    world.add_component(barrel_entity, Parent::new(turret_entity));
    world.add_component(barrel_entity, display::tank_barrel::TankBarrel::new());
    // world.add_component(barrel_entity, display::debug_lines::DebugLines::straight(10.0, 0.1, display::primitives::Color::BLUE));

    // -----   Nozzle
    world.add_component(nozzle_entity, Parent::new(barrel_entity));

    let cannon_config = components::cannon::CannonConfig {
        reload_time: 1.0,
        fire_effect: std::rc::Rc::new(cannon_function),
    };
    world.add_component(
        nozzle_entity,
        components::cannon::Cannon::new(cannon_config),
    );
    world.add_component(
        nozzle_entity,
        PreTransform::from_translation(Vec3::new(1.0, 0.0, 0.0)),
    );

    register_interface.get_mut().add_module(
        "cannon",
        MODULE_CANNON,
        components::cannon::CannonModule::new(nozzle_entity),
    );

    // -----   Radar
    let mut radar_revolute =
        components::revolute::Revolute::new_with_axis(Vec3::new(0.0, 0.0, 1.0));
    radar_revolute.set_velocity_bounds(-std::f32::consts::PI * 2.0, std::f32::consts::PI * 2.0);
    radar_revolute.set_velocity(-std::f32::consts::PI);
    register_interface.get_mut().add_module(
        "radar_rotation",
        MODULE_REVOLUTE_RADAR,
        components::revolute::RevoluteModule::new(radar_entity),
    );
    register_interface.get_mut().add_module(
        "radar",
        MODULE_RADAR,
        components::radar::RadarModule::new(radar_entity),
    );

    world.add_component(radar_entity, radar_revolute);
    // world.add_component(radar_entity, display::debug_lines::DebugLines::straight(10.0, 0.01, display::primitives::Color::RED));
    world.add_component(
        radar_entity,
        PreTransform::from_translation(Vec3::new(0.0, 0.0, 0.07)),
    );

    world.add_component(radar_entity, components::pose::Pose::new());
    world.add_component(radar_entity, components::velocity::Velocity::new());
    world.add_component(radar_entity, Parent::new(turret_entity));
    // world.add_component(radar_entity, display::debug_box::DebugBox::new(0.1, 0.1, 0.1));
    world.add_component(radar_entity, display::radar_model::RadarModel::new());
    world.add_component(
        radar_entity,
        components::radar::Radar::new_with_config(components::radar::RadarConfig {
            range_max: 30.0,
            detection_angle_yaw: 10.0f32.to_radians(),
            detection_angle_pitch: 180f32.to_radians(),
            // range_max: 70.0,
            // detection_angle_yaw: 45.0f32.to_radians(),
            // detection_angle_pitch: 180f32.to_radians(),
            signal_strength: 1.0,
        }),
    );

    // -----   Flag
    world.add_component(
        flag_entity,
        Pose::from_xyz(-0.8, -0.4, 0.3).rotated_angle_z(cgmath::Deg(180.0)),
    );
    world.add_component(
        flag_entity,
        display::flag::Flag::from_scale_color(0.5, display::Color::RED),
    );
    world.add_component(flag_entity, Parent::new(base_entity));

    // -----   Control
    world.add_component(control_entity, register_interface);

    // Finally, add the controller.
    let rc = components::unit_controller::UnitControlStorage::new(config.controller);
    world.add_component(
        control_entity,
        components::unit_controller::UnitController::new(rc),
    );

    // Add the group, unit and team membership to each of the component.
    let group = Group::from(&tank_group_entities);
    for e in tank_group_entities.iter() {
        world.add_component(*e, group.clone());
        world.add_component(*e, components::unit_member::UnitMember::new(unit_id));
        // This feels a bit like a crux... but it's cheap and easy.
        if let Some(team_member) = config.team_member {
            world.add_component(*e, team_member);
        }
    }

    unit_entity
}

pub fn cannon_function(world: &mut World, cannon_entity: EntityId) {
    use crate::components::point_projectile::PointProjectile;
    use crate::components::unit_source::UnitSource;
    use crate::components::velocity::Velocity;

    let muzzle_pose = components::pose::world_pose(world, cannon_entity);
    let muzzle_world_velocity = components::velocity::world_velocity(world, cannon_entity);

    // Get the unit source of this cannel.

    let muzzle_velocity = 10.0;

    // Get the pose of the cannon in the world coordinates. Then create the pose with the
    // Orientation in the global frame.
    let projectile_entity = world.add_entity();
    world.add_component::<PointProjectile>(projectile_entity, PointProjectile::new());
    let unit_id = world
        .component::<components::unit_member::UnitMember>(cannon_entity)
        .map(|v| v.unit());
    if let Some(unit_member) = unit_id {
        world.add_component(projectile_entity, UnitSource::new(unit_member));
    }
    world.add_component::<Pose>(
        projectile_entity,
        Pose::from_mat4(cgmath::Matrix4::<f32>::from_translation(
            muzzle_pose.w.truncate(),
        )),
    );

    // Calculate the velocity vector in the global frame.
    let mut muzzle_pose = *muzzle_pose.transform();
    // zero out the translation components.
    muzzle_pose.w[0] = 0.0;
    muzzle_pose.w[1] = 0.0;
    let v = muzzle_pose * cgmath::Vector4::<f32>::new(muzzle_velocity, 0.0, 0.0, 1.0);
    let v = v + muzzle_world_velocity.v.extend(0.0);
    let projectile_velocity =
        Velocity::from_velocities(v.truncate(), cgmath::Vector3::<f32>::new(0.0, 0.0, 0.0));

    // And add the velocity to the projectile.
    world.add_component::<Velocity>(projectile_entity, projectile_velocity);
    // world.add_component(projectile_entity, crate::display::debug_box::DebugBox::from_size(0.2));
    world.add_component(
        projectile_entity,
        crate::display::tank_bullet::TankBullet::new(),
    );

    // Clearly not the place for this to be... but works for now.
    world.add_component(
        projectile_entity,
        crate::components::acceleration::Acceleration::gravity(),
    );
    world.add_component(
        projectile_entity,
        components::damage_hit::DamageHit::new(3330.3),
    );

    world.add_component(
        projectile_entity,
        components::hit_effect::HitEffect::new(std::rc::Rc::new(cannon_hit_effect)),
    );

    let effect_entity = components::id_generator::generate_id(world);
    world.add_component(
        projectile_entity,
        crate::display::particle_emitter::ParticleEmitter::bullet_trail(
            effect_entity,
            0.05,
            crate::display::Color::WHITE,
        ),
    );
}

fn cannon_hit_effect(
    world: &mut World,
    projectile: EntityId,
    _impact: &components::impact::Impact,
) {
    // Create a bullet destructor.
    let projectile_destructor = world.add_entity();
    let effect_entity = components::id_generator::generate_id(world);
    let mut destructor = crate::display::deconstructor::Deconstructor::new(effect_entity);
    // destructor.add_impact(impact.position(), 0.005);
    destructor.add_element::<crate::display::tank_bullet::TankBullet>(projectile, world);
    world.add_component(projectile_destructor, destructor);
    world.add_component(
        projectile_destructor,
        crate::components::expiry::Expiry::lifetime(10.0),
    );
    // Now, we can remove the displayable mesh.
    world.remove_component::<display::tank_bullet::TankBullet>(projectile);

    // Copy the bullet to a new entity.
    let emitter_entity = world.add_entity();
    let emitter =
        world.remove_component::<crate::display::particle_emitter::ParticleEmitter>(emitter_entity);
    // Disable the particle emitter.
    if let Some(mut emitter) = emitter {
        emitter.emitting = false;
        world.add_component_boxed(emitter_entity, emitter);
    }

    world.add_component(
        emitter_entity,
        crate::components::expiry::Expiry::lifetime(5.0),
    );
}
