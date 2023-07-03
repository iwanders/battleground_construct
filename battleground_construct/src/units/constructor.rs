use super::Unit;
use crate::components;
use crate::display;
use crate::display::primitives::Vec3;
use components::group::Group;
use components::parent::Parent;
use components::pose::{Pose, PreTransform};
use engine::prelude::*;
use serde::{Deserialize, Serialize};

use battleground_unit_control::units::constructor::*;

const CONSTRUCTOR_RADAR_REFLECTIVITY: f32 = 0.5;


pub struct ConstructorSpawnConfig {
    pub x: f32,
    pub y: f32,
    pub yaw: f32,
    pub controller: Box<dyn battleground_unit_control::UnitControl>,
    pub team_member: Option<components::team_member::TeamMember>,
    pub radio_config: Option<super::common::RadioConfig>,
}

impl Default for ConstructorSpawnConfig {
    fn default() -> Self {
        ConstructorSpawnConfig {
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
pub struct UnitConstructor {
    pub unit_entity: EntityId,
    pub control_entity: EntityId,
    pub base_entity: EntityId,
    pub body_entity: EntityId,
    pub health_bar_entity: EntityId,
    pub flag_entity: EntityId,
}
impl Component for UnitConstructor {}

impl Unit for UnitConstructor {
    fn children(&self) -> Vec<EntityId> {
        vec![
            self.control_entity,
            self.base_entity,
            self.body_entity,
            self.health_bar_entity,
            self.flag_entity,
        ]
    }
}

/// Spawn a constructor, returning the unit entity.
pub fn spawn_constructor(world: &mut World, config: ConstructorSpawnConfig) -> EntityId {
    /*
        Topology of the constructor;

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

        The Unit and Control entities are 'free'.
        Base to Barrel forms a chain of Parent, all entities are part of the group.
    */
    let unit_entity = world.add_entity();
    let control_entity = world.add_entity();

    let base_entity = world.add_entity();
    let body_entity = world.add_entity();
    let flag_entity = world.add_entity();
    let health_bar_entity = world.add_entity();

    let unit_constructor = UnitConstructor {
        unit_entity,
        control_entity,
        base_entity,
        body_entity,
        flag_entity,
        health_bar_entity,
    };
    // Unit must be first in the group!
    let mut constructor_group_entities: Vec<EntityId> = vec![unit_entity];
    constructor_group_entities.append(&mut unit_constructor.children());

    // Create the register interface, we'll add modules throughout this function.
    let register_interface = components::unit_interface::RegisterInterfaceContainer::new(
        components::unit_interface::RegisterInterface::new(),
    );
    super::common::add_common_global(&register_interface);

    world.add_component(unit_entity, unit_constructor);

    let unit_id = super::common::add_common_unit(
        world,
        &register_interface,
        unit_entity,
        battleground_unit_control::units::UnitType::Constructor,
    );

    add_constructor_passive(world, &unit_constructor);

    // -----   Base
    world.add_component(base_entity, Pose::from_se2(config.x, config.y, config.yaw));
    // let diff_drive_config = components::differential_drive_base::DifferentialDriveConfig {
        // track_width: 1.0,
        // wheel_velocity_bounds: (-1.0, 1.0),
        // wheel_acceleration_bounds: Some((-0.5, 0.5)),
    // };
    // super::common::add_common_diff_drive(
        // world,
        // &register_interface,
        // base_entity,
        // diff_drive_config,
        // MODULE_CONSTRUCTOR_DIFF_DRIVE,
    // );

    // -----   Body
    // const CONSTRUCTOR_DIM_FLOOR_TO_BODY_Z: f32 = 0.3;
    world.add_component(body_entity, Parent::new(base_entity));
    world.add_component(
        body_entity,
        PreTransform::from_translation(Vec3::new(0.0, 0.0, CONSTRUCTOR_DIM_FLOOR_TO_BODY_Z)),
    );

    super::common::add_radio_receiver_transmitter(
        world,
        &register_interface,
        body_entity,
        config.radio_config,
    );
    super::common::add_common_body(
        world,
        &register_interface,
        CONSTRUCTOR_RADAR_REFLECTIVITY,
        body_entity,
    );


    // -----   Control
    world.add_component(control_entity, display::draw_module::DrawComponent::new());
    register_interface.get_mut().add_module(
        "draw",
        battleground_unit_control::units::common::MODULE_DRAW,
        display::draw_module::DrawModule::new(control_entity),
    );
    // Finally, add the controller.
    let rc = components::unit_controller::UnitControlStorage::new(config.controller);
    world.add_component(
        control_entity,
        components::unit_controller::UnitController::new(rc),
    );
    register_interface.get_mut().add_module(
        "controller",
        battleground_unit_control::units::common::MODULE_CONTROLLER,
        components::unit_controller::UnitControllerModule::new(control_entity),
    );
    world.add_component(control_entity, register_interface);

    // Add the group, unit and team membership to each of the component.
    // Unit must be first in the group!
    let mut constructor_group_entities: Vec<EntityId> = vec![unit_entity];
    constructor_group_entities.append(&mut unit_constructor.children());

    let group = Group::from(&constructor_group_entities);
    for e in constructor_group_entities.iter() {
        world.add_component(*e, group.clone());
        world.add_component(*e, components::unit_member::UnitMember::new(unit_id));
        // This feels a bit like a crux... but it's cheap and easy.
        if let Some(team_member) = config.team_member {
            world.add_component(*e, team_member);
        }
    }

    unit_entity
}

pub fn add_constructor_passive(world: &mut World, unit: &UnitConstructor) {
    // -----   Body
    let body = display::wheeled_body::WheeledBody::new();
    let hitbox = body.hitbox();
    world.add_component(unit.body_entity, body);
    world.add_component(
        unit.body_entity,
        components::select_box::SelectBox::from_hit_box(&hitbox),
    );
    world.add_component(unit.body_entity, hitbox);


    // -----   Tracks
    /*
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
    */



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


