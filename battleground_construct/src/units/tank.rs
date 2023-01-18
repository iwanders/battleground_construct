use super::Unit;
use crate::components;
use crate::display;
use crate::display::primitives::Vec3;
use components::group::Group;
use components::parent::Parent;
use components::pose::{Pose, PreTransform};
use engine::prelude::*;
use serde::{Deserialize, Serialize};

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

#[derive(Deserialize, Serialize, Debug, Clone, Copy)]
pub struct UnitTank {
    pub unit_entity: EntityId,
    pub control_entity: EntityId,
    pub base_entity: EntityId,
    pub body_entity: EntityId,
    pub turret_entity: EntityId,
    pub radar_entity: EntityId,
    pub health_bar_entity: EntityId,
    pub flag_entity: EntityId,
    pub barrel_entity: EntityId,
    pub muzzle_entity: EntityId,
}
impl Component for UnitTank {}

impl Unit for UnitTank {
    fn children(&self) -> Vec<EntityId> {
        vec![
            self.control_entity,
            self.base_entity,
            self.body_entity,
            self.turret_entity,
            self.radar_entity,
            self.health_bar_entity,
            self.flag_entity,
            self.barrel_entity,
            self.muzzle_entity,
        ]
    }
}

/// Spawn a tank, returning the unit entity.
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
            -> Health Bar entity
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
    let health_bar_entity = world.add_entity();
    let barrel_entity = world.add_entity();
    let muzzle_entity = world.add_entity();

    let unit_tank = UnitTank {
        unit_entity,
        control_entity,
        base_entity,
        body_entity,
        turret_entity,
        radar_entity,
        flag_entity,
        health_bar_entity,
        barrel_entity,
        muzzle_entity,
    };
    // Unit must be first in the group!
    let mut tank_group_entities: Vec<EntityId> = vec![unit_entity];
    tank_group_entities.append(&mut unit_tank.children());

    // Create the register interface, we'll add modules throughout this function.
    let register_interface = components::unit_interface::RegisterInterfaceContainer::new(
        components::unit_interface::RegisterInterface::new(),
    );
    super::common::add_common_global(&register_interface);

    world.add_component(unit_entity, unit_tank);

    let unit_id = super::common::add_common_unit(
        world,
        &register_interface,
        unit_entity,
        battleground_unit_control::units::UnitType::Tank,
    );

    add_tank_passive(world, &unit_tank);

    // -----   Base
    world.add_component(base_entity, Pose::from_se2(config.x, config.y, config.yaw));
    let diff_drive_config = components::differential_drive_base::DifferentialDriveConfig {
        track_width: 1.0,
        wheel_velocity_bounds: (-1.0, 1.0),
        wheel_acceleration_bounds: Some((-0.5, 0.5)),
    };
    super::common::add_common_diff_drive(
        world,
        &register_interface,
        base_entity,
        diff_drive_config,
        MODULE_TANK_DIFF_DRIVE,
    );

    // -----   Body
    world.add_component(body_entity, Parent::new(base_entity));
    world.add_component(
        body_entity,
        PreTransform::from_translation(Vec3::new(0.0, 0.0, TANK_DIM_FLOOR_TO_BODY_Z)),
    );

    super::common::add_radio_receiver_transmitter(
        world,
        &register_interface,
        body_entity,
        config.radio_config,
    );
    super::common::add_common_body(world, &register_interface, 0.5, body_entity);

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
        MODULE_TANK_REVOLUTE_TURRET,
        revolute_config,
    );
    world.add_component(
        turret_entity,
        PreTransform::from_translation(Vec3::new(0.0, 0.0, TANK_DIM_FLOOR_TO_TURRET_Z)),
    );
    world.add_component(turret_entity, Parent::new(base_entity));

    // -----   Barrel
    let revolute_config = components::revolute::RevoluteConfig {
        axis: Vec3::new(0.0, 1.0, 0.0),
        velocity_bounds: (-1.0, 1.0),
        acceleration_bounds: Some((-2.0, 2.0)),
        ..Default::default()
    };
    super::common::add_revolute(
        world,
        &register_interface,
        barrel_entity,
        "barrel",
        MODULE_TANK_REVOLUTE_BARREL,
        revolute_config,
    );
    world.add_component(
        barrel_entity,
        PreTransform::from_translation(Vec3::new(TANK_DIM_TURRET_TO_BARREL_X, 0.0, 0.0)),
    );
    world.add_component(barrel_entity, Parent::new(turret_entity));

    // -----   Muzzle
    world.add_component(muzzle_entity, Parent::new(barrel_entity));

    let cannon_config = components::cannon::CannonConfig {
        reload_time: 2.0,
        fire_effect: std::rc::Rc::new(cannon_function),
    };
    world.add_component(
        muzzle_entity,
        components::cannon::Cannon::new(cannon_config),
    );
    world.add_component(
        muzzle_entity,
        PreTransform::from_translation(Vec3::new(TANK_DIM_BARREL_TO_MUZZLE_X, 0.0, 0.0)),
    );
    // world.add_component(muzzle_entity, display::debug_box::DebugBox::cube(0.1));

    register_interface.get_mut().add_module(
        "cannon",
        MODULE_TANK_CANNON,
        components::cannon::CannonModule::new(muzzle_entity),
    );

    // -----   Radar
    world.add_component(radar_entity, Parent::new(turret_entity));

    world.add_component(
        radar_entity,
        PreTransform::from_translation(Vec3::new(0.0, 0.0, TANK_DIM_TURRET_TO_RADAR_Z)),
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
        radar_entity,
        "radar_rotation",
        MODULE_TANK_REVOLUTE_RADAR,
        revolute_config,
    );
    let radar_config = components::radar::RadarConfig {
        range_max: 20.0,
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
        MODULE_TANK_RADAR,
        radar_config,
    );

    // -----   Control
    world.add_component(control_entity, display::draw_module::DrawComponent::new());
    register_interface.get_mut().add_module(
        "draw",
        battleground_unit_control::units::common::MODULE_DRAW,
        display::draw_module::DrawModule::new(control_entity),
    );
    world.add_component(control_entity, register_interface);

    // Finally, add the controller.
    let rc = components::unit_controller::UnitControlStorage::new(config.controller);
    world.add_component(
        control_entity,
        components::unit_controller::UnitController::new(rc),
    );

    // Add the group, unit and team membership to each of the component.
    // Unit must be first in the group!
    let mut tank_group_entities: Vec<EntityId> = vec![unit_entity];
    tank_group_entities.append(&mut unit_tank.children());

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

pub fn add_tank_passive(world: &mut World, unit: &UnitTank) {
    // -----   Body
    let body = display::tank_body::TankBody::new();
    let hitbox = body.hitbox();
    world.add_component(unit.body_entity, body);
    world.add_component(
        unit.body_entity,
        components::select_box::SelectBox::from_hit_box(&hitbox),
    );
    world.add_component(unit.body_entity, hitbox);

    // -----   Turrent
    let tank_turret = display::tank_turret::TankTurret::new();
    let hitbox = tank_turret.hitbox();
    world.add_component(unit.turret_entity, hitbox);
    world.add_component(
        unit.turret_entity,
        components::select_box::SelectBox::from_hit_box(&hitbox),
    );
    world.add_component(unit.turret_entity, tank_turret);

    // -----   Tracks
    let track_config = display::tracks_side::TracksSideConfig {
        width: 0.4,
        length: 1.4,
        height: 0.2,
        track_width: 1.0,
    };
    let tracks = display::tracks_side::TracksSide::from_config(track_config, unit.base_entity);
    let hit_collection =
        components::hit_collection::HitCollection::from_hit_boxes(&tracks.hit_boxes());
    world.add_component(unit.base_entity, tracks);
    world.add_component(unit.base_entity, hit_collection);

    // -----   Barrel
    let tank_barrel = display::tank_barrel::TankBarrel::new();
    let hit_collection =
        components::hit_collection::HitCollection::from_hit_boxes(&tank_barrel.hit_boxes());
    world.add_component(unit.barrel_entity, hit_collection);
    world.add_component(unit.barrel_entity, tank_barrel);

    // -----   Radar
    world.add_component(unit.radar_entity, display::radar_model::RadarModel::new());

    // -----   Flag
    world.add_component(
        unit.flag_entity,
        display::flag::Flag::from_scale_color(0.5, display::Color::RED),
    );
    world.add_component(
        unit.flag_entity,
        Pose::from_xyz(-0.8, -0.4, 0.3).rotated_angle_z(cgmath::Deg(180.0)),
    );
    world.add_component(unit.flag_entity, Parent::new(unit.base_entity));

    // -----   Health Bar
    world.add_component(
        unit.health_bar_entity,
        Pose::from_xyz(-0.8, 0.0, 0.40).rotated_angle_z(cgmath::Deg(90.0)),
    );
    world.add_component(
        unit.health_bar_entity,
        display::health_bar::HealthBar::new(unit.unit_entity, 0.6),
    );
    world.add_component(unit.health_bar_entity, Parent::new(unit.base_entity));
}

pub fn cannon_function(world: &mut World, cannon_entity: EntityId) {
    use crate::components::point_projectile::PointProjectile;
    use crate::components::unit_source::UnitSource;
    use crate::components::velocity::Velocity;

    let muzzle_pose_raw = components::pose::world_pose(world, cannon_entity);
    let muzzle_pose = muzzle_pose_raw;
    // println!("muzzle_pose: {muzzle_pose:?}");
    let muzzle_world_velocity = components::velocity::world_velocity(world, cannon_entity);

    // Get the unit source of this cannel.

    let muzzle_velocity = TANK_PARAM_MUZZLE_VELOCITY;

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
        components::damage_hit::DamageHit::new(0.3),
    );

    world.add_component(
        projectile_entity,
        components::hit_effect::HitEffect::new(std::rc::Rc::new(cannon_hit_effect)),
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
    world.add_component(emitter_entity, muzzle_pose_raw);
}

fn cannon_hit_effect(
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
