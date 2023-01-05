use crate::components;
use crate::display;
use crate::display::primitives::{Mat4, Vec3};
use components::group::Group;
use components::parent::Parent;
use components::pose::{Pose, PreTransform};
use engine::prelude::*;

use battleground_unit_control::units::artillery::*;

pub struct ArtillerySpawnConfig {
    pub x: f32,
    pub y: f32,
    pub yaw: f32,
    pub controller: Box<dyn battleground_unit_control::UnitControl>,
    pub team_member: Option<components::team_member::TeamMember>,
    pub radio_config: Option<super::common::RadioConfig>,
}

impl Default for ArtillerySpawnConfig {
    fn default() -> Self {
        ArtillerySpawnConfig {
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
pub struct UnitArtillery {
    pub unit_entity: EntityId,
    pub control_entity: EntityId,
    pub base_entity: EntityId,
    pub front_track_entity: EntityId,
    pub rear_track_entity: EntityId,
    pub body_entity: EntityId,
    pub turret_entity: EntityId,
    pub radar_joint_entity: EntityId,
    pub radar_entity: EntityId,
    pub flag_entity: EntityId,
    pub barrel_entity: EntityId,
    pub muzzle_entity: EntityId,
}
impl Component for UnitArtillery {}

/// Spawn a artillery, returning the unit entity.
pub fn spawn_artillery(world: &mut World, config: ArtillerySpawnConfig) -> EntityId {
    /*
        Topology of the artillery;

        Unit Entity:
            - Health
            - TeamMember
            - Eternal

        Control Entity:
            - UnitController

        Base Entity:
            - Diff Drive controller
            -> Front Track
            -> Rear Track
            -> Body entity
                - RadarReflector
                - CaptureMarker
                - Radio's
            -> Flag entity
            -> Turret Entity
                - Revolute
                -> Barrel Entity
                    -> Muzzle Entity
                -> Radar joint
                    -> Radar entity
                        - Radar

        The Unit and Control entities are 'free'.
        Base to Barrel forms a chain of Parent, all entities are part of the group.
    */
    let unit_entity = world.add_entity();
    let control_entity = world.add_entity();

    let base_entity = world.add_entity();
    let front_track_entity = world.add_entity();
    let rear_track_entity = world.add_entity();
    let body_entity = world.add_entity();
    let turret_entity = world.add_entity();
    let radar_joint_entity = world.add_entity();
    let radar_entity = world.add_entity();
    let flag_entity = world.add_entity();
    let barrel_entity = world.add_entity();
    let muzzle_entity = world.add_entity();

    let artillery_group_entities: Vec<EntityId> = vec![
        unit_entity,
        control_entity,
        base_entity,
        front_track_entity,
        rear_track_entity,
        body_entity,
        turret_entity,
        radar_joint_entity,
        radar_entity,
        flag_entity,
        barrel_entity,
        muzzle_entity,
    ];
    let unit_artillery = UnitArtillery {
        unit_entity,
        control_entity,
        base_entity,
        front_track_entity,
        rear_track_entity,
        body_entity,
        turret_entity,
        radar_joint_entity,
        radar_entity,
        flag_entity,
        barrel_entity,
        muzzle_entity,
    };

    // Create the register interface, we'll add modules throughout this function.
    let register_interface = components::unit_interface::RegisterInterfaceContainer::new(
        components::unit_interface::RegisterInterface::new(),
    );
    super::common::add_common_global(&register_interface);

    let unit_id = super::common::add_common_unit(
        world,
        &register_interface,
        unit_entity,
        battleground_unit_control::units::UnitType::Artillery,
    );

    world.add_component(unit_entity, unit_artillery);

    // -----   Base

    world.add_component(base_entity, Pose::from_se2(config.x, config.y, config.yaw));
    let track_width = 1.75;
    let diff_drive_config = components::differential_drive_base::DifferentialDriveConfig {
        track_width: track_width,
        wheel_velocity_bounds: (-1.0, 1.0),
        wheel_acceleration_bounds: Some((-0.5, 0.5)),
    };
    super::common::add_common_diff_drive(
        world,
        &register_interface,
        base_entity,
        diff_drive_config,
        MODULE_ARTILLERY_DIFF_DRIVE,
    );

    let track_config = display::tracks_side::TracksSideConfig {
        width: 0.2,
        length: 1.0,
        height: 0.2,
        track_width: track_width,
    };

    world.add_component(front_track_entity, Parent::new(base_entity));
    world.add_component(front_track_entity, PreTransform::from_se2(0.75, 0.0, 0.0));
    let tracks = display::tracks_side::TracksSide::from_config(track_config, base_entity);
    world.add_component(front_track_entity, tracks);
    let hit_collection =
        components::hit_collection::HitCollection::from_hit_boxes(&tracks.hit_boxes());
    world.add_component(front_track_entity, hit_collection);

    // Second track set.
    world.add_component(rear_track_entity, Parent::new(base_entity));
    world.add_component(rear_track_entity, PreTransform::from_se2(-0.75, 0.0, 0.0));
    let tracks = display::tracks_side::TracksSide::from_config(track_config, base_entity);
    world.add_component(rear_track_entity, tracks);
    let hit_collection =
        components::hit_collection::HitCollection::from_hit_boxes(&tracks.hit_boxes());
    world.add_component(rear_track_entity, hit_collection);

    // world.add_component(base_entity, display::artillery_tracks::ArtilleryTracks::new());

    // -----   Body
    world.add_component(body_entity, Parent::new(base_entity));
    world.add_component(
        body_entity,
        PreTransform::from_translation(Vec3::new(0.0, 0.0, ARTILLERY_DIM_FLOOR_TO_BODY_Z)),
    );
    let body = display::artillery_body::ArtilleryBody::new();

    let hitbox = body.hitbox();
    world.add_component(body_entity, body);
    world.add_component(
        body_entity,
        components::select_box::SelectBox::from_hit_box(&hitbox),
    );
    let hit_collection = components::hit_collection::HitCollection::from_hit_box(hitbox);
    world.add_component(body_entity, hit_collection);

    super::common::add_radio_receiver_transmitter(
        world,
        &register_interface,
        body_entity,
        config.radio_config,
    );
    super::common::add_common_body(world, &register_interface, body_entity);

    // -----   Turret
    let revolute_config = components::revolute::RevoluteConfig {
        axis: Vec3::new(0.0, 0.0, 1.0),
        velocity_bounds: (-1.0, 1.0),
        acceleration_bounds: Some((-1.0, 1.0)),
        ..Default::default()
    };
    super::common::add_revolute(
        world,
        &register_interface,
        turret_entity,
        "turret",
        MODULE_ARTILLERY_REVOLUTE_TURRET,
        revolute_config,
    );
    world.add_component(
        turret_entity,
        PreTransform::from_translation(Vec3::new(0.0, 0.0, ARTILLERY_DIM_FLOOR_TO_TURRET_Z)),
    );
    world.add_component(turret_entity, Parent::new(base_entity));
    world.add_component(
        turret_entity,
        display::artillery_turret::ArtilleryTurret::new(),
    );

    // -----   Barrel
    let revolute_config = components::revolute::RevoluteConfig {
        axis: Vec3::new(0.0, 1.0, 0.0),
        velocity_bounds: (-1.0, 1.0),
        acceleration_bounds: Some((-2.0, 2.0)),
        velocity_cmd: 0.3,
        ..Default::default()
    };
    super::common::add_revolute(
        world,
        &register_interface,
        barrel_entity,
        "barrel",
        MODULE_ARTILLERY_REVOLUTE_BARREL,
        revolute_config,
    );
    world.add_component(
        barrel_entity,
        PreTransform::from_translation(Vec3::new(0.0, 0.0, ARTILLERY_DIM_TURRET_TO_BARREL_Z)),
    );
    let artillery_barrel = display::artillery_barrel::ArtilleryBarrel::new();

    let hitbox = artillery_barrel.hitbox();
    world.add_component(
        barrel_entity,
        components::select_box::SelectBox::from_hit_box(&hitbox),
    );
    let hit_collection = components::hit_collection::HitCollection::from_hit_box(hitbox);
    world.add_component(body_entity, hit_collection);

    world.add_component(barrel_entity, Parent::new(turret_entity));

    world.add_component(barrel_entity, artillery_barrel);

    // -----   Muzzle
    world.add_component(muzzle_entity, Parent::new(barrel_entity));
    world.add_component(
        muzzle_entity,
        PreTransform::from_translation(Vec3::new(ARTILLERY_DIM_BARREL_TO_MUZZLE_X, 0.0, 0.0)),
    );
    // world.add_component(muzzle_entity, display::debug_box::DebugBox::cube(0.1));

    let inter_gun_duration = 1.0;
    let gun_reload = 2.0;
    let battery_reload = 3.0;
    let gun_battery_config = components::gun_battery::GunBatteryConfig {
        fire_effect: std::rc::Rc::new(artillery_fire_function),
        inter_gun_duration,
        gun_reload,
        battery_reload,
        poses: vec![
            Mat4::from_translation(Vec3::new(0.0, 0.0, 0.0)),
            Mat4::from_translation(Vec3::new(0.0, 1.0, 0.0)),
            Mat4::from_translation(Vec3::new(0.0, 2.0, 0.0)),
            Mat4::from_translation(Vec3::new(0.0, 3.0, 0.0)),
        ],
    };
    world.add_component(
        muzzle_entity,
        components::gun_battery::GunBattery::new(gun_battery_config),
    );
    // register_interface.get_mut().add_module(
    // "cannon",
    // MODULE_ARTILLERY_CANNON,
    // components::cannon::CannonModule::new(muzzle_entity),
    // );
    /**/

    // -----   Radar
    world.add_component(radar_joint_entity, Parent::new(turret_entity));

    world.add_component(
        radar_joint_entity,
        PreTransform::from_translation(Vec3::new(0.0, 0.0, ARTILLERY_DIM_TURRET_TO_RADAR_Z)),
    );

    let revolute_config = components::revolute::RevoluteConfig {
        axis: Vec3::new(0.0, 0.0, 1.0),
        velocity_bounds: (-std::f32::consts::PI * 2.0, std::f32::consts::PI * 2.0),
        velocity_cmd: -std::f32::consts::PI,
        acceleration_bounds: Some((-std::f32::consts::PI, std::f32::consts::PI)),
        ..Default::default()
    };
    super::common::add_revolute(
        world,
        &register_interface,
        radar_joint_entity,
        "radar_rotation",
        MODULE_ARTILLERY_REVOLUTE_RADAR,
        revolute_config,
    );

    world.add_component(radar_entity, Parent::new(radar_joint_entity));

    world.add_component(
        radar_entity,
        PreTransform::from_translation(Vec3::new(ARTILLERY_DIM_RADAR_JOINT_TO_RADAR_X, 0.0, 0.0)),
    );

    let radar_config = components::radar::RadarConfig {
        range_max: 30.0,
        detection_angle_yaw: 10.0f32.to_radians(),
        detection_angle_pitch: 180f32.to_radians(),
        // range_max: 70.0,
        // detection_angle_yaw: 45.0f32.to_radians(),
        // detection_angle_pitch: 180f32.to_radians(),
        signal_strength: 1.0,
    };
    super::common::add_radar(
        world,
        &register_interface,
        radar_entity,
        "radar",
        MODULE_ARTILLERY_RADAR,
        radar_config,
    );
    world.add_component(radar_entity, display::radar_model::RadarModel::new());

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
    let group = Group::from(&artillery_group_entities);
    for e in artillery_group_entities.iter() {
        world.add_component(*e, group.clone());
        world.add_component(*e, components::unit_member::UnitMember::new(unit_id));
        // This feels a bit like a crux... but it's cheap and easy.
        if let Some(team_member) = config.team_member {
            world.add_component(*e, team_member);
        }
    }

    unit_entity
}

pub fn artillery_fire_function(world: &mut World, gun_battery_entity: EntityId, gun_pose: Mat4) {
    use crate::components::point_projectile::PointProjectile;
    use crate::components::unit_source::UnitSource;
    use crate::components::velocity::Velocity;

    let muzzle_pose_raw = components::pose::world_pose(world, gun_battery_entity);
    let muzzle_pose = muzzle_pose_raw;
    // println!("muzzle_pose: {muzzle_pose:?}");
    let muzzle_world_velocity = components::velocity::world_velocity(world, gun_battery_entity);

    // Get the unit source of this cannel.

    let muzzle_velocity = 10.0;

    // Get the pose of the cannon in the world coordinates. Then create the pose with the
    // Orientation in the global frame.
    let projectile_entity = world.add_entity();
    world.add_component::<PointProjectile>(projectile_entity, PointProjectile::new());
    let unit_id = world
        .component::<components::unit_member::UnitMember>(gun_battery_entity)
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
        components::hit_effect::HitEffect::new(std::rc::Rc::new(artillery_hit_effect)),
    );

    let effect_id = components::id_generator::generate_id(world);
    world.add_component(
        projectile_entity,
        crate::display::particle_emitter::ParticleEmitter::bullet_trail(
            effect_id,
            0.05,
            crate::display::Color::WHITE,
        ),
    );

    // Create an entity for the muzzle flash
    let emitter_entity = world.add_entity();
    let effect_id = components::id_generator::generate_id(world);
    world.add_component(
        emitter_entity,
        crate::display::particle_emitter::ParticleEmitter::muzzle_flash(
            effect_id,
            0.03,
            crate::display::Color::rgb(20, 20, 20),
        ),
    );
    world.add_component(
        emitter_entity,
        crate::components::expiry::Expiry::lifetime(15.0),
    );
    world.add_component(emitter_entity, muzzle_pose_raw.clone());
}

fn artillery_hit_effect(
    world: &mut World,
    projectile: EntityId,
    _impact: &components::impact::Impact,
) {
    // Create a bullet destructor.
    let projectile_destructor = world.add_entity();

    /*
    let mut destructor = crate::display::deconstructor::Deconstructor::new(effect_entity);
    // destructor.add_impact(impact.position(), 0.005);
    destructor.add_element::<crate::display::tank_bullet::TankBullet>(projectile, world);
    world.add_component(projectile_destructor, destructor);
    */
    let effect_id = components::id_generator::generate_id(world);
    let world_pose = crate::components::pose::world_pose(world, projectile);
    let world_vel = crate::components::velocity::world_velocity(world, projectile).to_twist();
    world.add_component(
        projectile_destructor,
        crate::display::particle_emitter::ParticleEmitter::bullet_impact(
            effect_id,
            0.03,
            crate::display::Color::BLACK,
            world_vel.v,
        ),
    );
    world.add_component(projectile_destructor, world_pose);
    world.add_component(
        projectile_destructor,
        crate::components::expiry::Expiry::lifetime(10.0),
    );
    // Now, we can remove the displayable mesh.
    world.remove_component::<display::tank_bullet::TankBullet>(projectile);

    // Copy the bullet to a new entity.
    let emitter_entity = world.add_entity();
    let emitter =
        world.remove_component::<crate::display::particle_emitter::ParticleEmitter>(projectile);
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
