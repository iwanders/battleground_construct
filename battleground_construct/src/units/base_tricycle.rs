use super::{Unit, UnitId};
use crate::components;
use crate::display;
use crate::display::primitives::Vec3;
use battleground_unit_control::units::UnitType;
use components::parent::Parent;
use components::pose::{Pose, PreTransform};
use engine::prelude::*;
use serde::{Deserialize, Serialize};

use battleground_unit_control::units::constructor::*;

// Display submodule doesn't use these.
const BASE_TRICYCLE_RADAR_REFLECTIVITY: f32 = 0.5;
const BASE_TRICYCLE_WHEEL_RADIUS: f32 = 0.15;
const BASE_TRICYCLE_WHEEL_WIDTH: f32 = 0.125;
const BASE_TRICYCLE_FRONT_WHEEL_OFFSET: f32 = 0.15;

#[derive(Deserialize, Serialize, Debug, Clone, Copy)]
pub struct BaseTricycle {
    pub unit_id: UnitId,
    pub unit_entity: EntityId,
    pub control_entity: EntityId,
    pub base_entity: EntityId,
    pub body_entity: EntityId,

    pub center_entity: EntityId,
    pub cabin_entity: EntityId,

    pub health_bar_entity: EntityId,
    pub flag_entity: EntityId,

    pub rear_left_wheel_entity: EntityId,
    pub rear_right_wheel_entity: EntityId,

    pub front_left_steer_entity: EntityId,
    pub front_right_steer_entity: EntityId,

    pub front_left_wheel_entity: EntityId,
    pub front_right_wheel_entity: EntityId,
}
impl Component for BaseTricycle {}

pub struct BaseTricycleSpawnConfig {
    pub x: f32,
    pub y: f32,
    pub yaw: f32,
    pub controller: Box<dyn battleground_unit_control::UnitControl>,
    pub team_member: Option<components::team_member::TeamMember>,
    pub radio_config: Option<super::common::RadioConfig>,
}

impl Unit for BaseTricycle {
    fn children(&self) -> Vec<EntityId> {
        vec![
            self.control_entity,
            self.base_entity,
            self.body_entity,
            self.center_entity,
            self.cabin_entity,
            self.health_bar_entity,
            self.flag_entity,
            self.rear_left_wheel_entity,
            self.rear_right_wheel_entity,
            self.front_left_wheel_entity,
            self.front_right_wheel_entity,
        ]
    }
    fn unit_entity(&self) -> EntityId {
        self.unit_entity
    }
    fn unit_id(&self) -> UnitId {
        self.unit_id
    }
}

/// Spawn a constructor, returning the unit entity.
pub fn spawn_base_tricycle(
    world: &mut World,
    config: BaseTricycleSpawnConfig,
    unit: UnitType,
) -> BaseTricycle {
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
    let center_entity = world.add_entity();
    let cabin_entity = world.add_entity();
    let flag_entity = world.add_entity();
    let health_bar_entity = world.add_entity();

    let rear_left_wheel_entity = world.add_entity();
    let rear_right_wheel_entity = world.add_entity();

    let front_left_steer_entity = world.add_entity();
    let front_right_steer_entity = world.add_entity();

    let front_left_wheel_entity = world.add_entity();
    let front_right_wheel_entity = world.add_entity();

    // Create the register interface, we'll add modules throughout this function.
    let register_interface = components::unit_interface::RegisterInterfaceContainer::new(
        components::unit_interface::RegisterInterface::new(),
    );
    let unit_id = super::common::add_common_unit(world, &register_interface, unit_entity, unit);

    let base_tricycle = BaseTricycle {
        unit_id,
        unit_entity,
        control_entity,
        base_entity,
        body_entity,
        center_entity,
        cabin_entity,
        flag_entity,
        health_bar_entity,

        rear_left_wheel_entity,
        rear_right_wheel_entity,

        front_left_wheel_entity,
        front_right_wheel_entity,

        front_left_steer_entity,
        front_right_steer_entity,
    };
    // Unit must be first in the group!
    let mut constructor_group_entities: Vec<EntityId> = vec![unit_entity];
    constructor_group_entities.append(&mut base_tricycle.children());

    super::common::add_common_global(&register_interface);

    world.add_component(unit_entity, base_tricycle);

    add_base_tricycle_passive(world, &base_tricycle);

    // -----   Base
    let body = display::wheeled_body::WheeledBody::new();

    world.add_component(base_entity, Pose::from_se2(config.x, config.y, config.yaw));
    let tricycle_config = components::tricycle_base::TricycleConfig {
        wheel_base: BASE_TRICYCLE_WHEEL_BASE,
        wheel_velocity_bounds: (-1.0, 1.0),
        wheel_acceleration_bounds: Some((-0.5, 0.5)),
    };
    super::common::add_common_tricycle(
        world,
        &register_interface,
        base_entity,
        front_left_steer_entity,
        tricycle_config,
        MODULE_BASE_TRICYCLE_DRIVE,
    );
    world.add_component(
        base_entity,
        components::tricycle_front_wheels::TricycleFrontWheels::new(&[
            front_left_wheel_entity,
            front_right_wheel_entity,
        ]),
    );
    world.add_component(
        base_entity,
        components::tricycle_rear_wheels::TricycleRearWheels::new(
            &[rear_left_wheel_entity, rear_right_wheel_entity],
            body.track_width(),
        ),
    );

    // -----   Body
    world.add_component(body_entity, Parent::new(base_entity));
    world.add_component(
        body_entity,
        PreTransform::from_translation(Vec3::new(0.0, 0.0, BASE_TRICYCLE_DIM_FLOOR_TO_BODY_Z)),
    );

    // ---- Center
    world.add_component(center_entity, Parent::new(body_entity));
    world.add_component(
        center_entity,
        PreTransform::from_translation(Vec3::new(body.center_offset(), 0.0, 0.0)),
    );

    // ---- Cabin
    world.add_component(cabin_entity, Parent::new(body_entity));
    world.add_component(
        cabin_entity,
        PreTransform::from_translation(body.cabin_offset()),
    );

    super::common::add_radio_receiver_transmitter(
        world,
        &register_interface,
        center_entity,
        config.radio_config,
    );
    super::common::add_common_body(
        world,
        &register_interface,
        BASE_TRICYCLE_RADAR_REFLECTIVITY,
        center_entity,
    );

    // -----   Wheels

    world.add_component(rear_left_wheel_entity, Parent::new(body_entity));
    world.add_component(
        rear_left_wheel_entity,
        PreTransform::from_mat4(*body.pose_rear_left_wheel()),
    );
    world.add_component(rear_right_wheel_entity, Parent::new(body_entity));
    world.add_component(
        rear_right_wheel_entity,
        PreTransform::from_mat4(*body.pose_rear_right_wheel()),
    );

    world.add_component(front_left_steer_entity, Parent::new(body_entity));
    world.add_component(
        front_left_steer_entity,
        PreTransform::from_mat4(*body.pose_front_left_wheel()),
    );
    world.add_component(front_right_steer_entity, Parent::new(body_entity));
    world.add_component(
        front_right_steer_entity,
        PreTransform::from_mat4(*body.pose_front_right_wheel()),
    );

    let revolute_config = components::revolute::RevoluteConfig {
        axis: Vec3::new(0.0, 0.0, 1.0),
        velocity_bounds: (-1.0, 1.0),
        acceleration_bounds: Some((-1.0, 1.0)),
        velocity_cmd: 0.1,
        ..Default::default()
    };
    super::common::add_revolute(
        world,
        &register_interface,
        front_left_steer_entity,
        "steer",
        MODULE_BASE_TRICYCLE_REVOLUTE_STEER,
        revolute_config,
    );
    world.add_component(
        front_left_wheel_entity,
        Parent::new(front_left_steer_entity),
    );
    world.add_component(
        front_left_wheel_entity,
        PreTransform::from_translation(Vec3::new(-BASE_TRICYCLE_FRONT_WHEEL_OFFSET, 0.0, 0.0)),
    );

    super::common::add_revolute_pair(world, front_right_steer_entity, front_left_steer_entity);

    world.add_component(
        front_right_wheel_entity,
        Parent::new(front_right_steer_entity),
    );
    world.add_component(
        front_right_wheel_entity,
        PreTransform::from_translation(Vec3::new(BASE_TRICYCLE_FRONT_WHEEL_OFFSET, 0.0, 0.0)),
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

    base_tricycle
}

pub fn add_base_tricycle_passive(world: &mut World, unit: &BaseTricycle) {
    // -----   Body
    let body = display::wheeled_body::WheeledBody::new();
    world.add_component(unit.body_entity, body);

    // ----- Center entity
    let hitbox = body.hitbox();
    world.add_component(
        unit.center_entity,
        components::select_box::SelectBox::from_hit_box(&hitbox),
    );
    world.add_component(unit.center_entity, hitbox);

    // ---- Cabin entity:
    let cabin_hitbox = body.cabin_hitbox();
    world.add_component(
        unit.cabin_entity,
        components::select_box::SelectBox::from_hit_box(&cabin_hitbox),
    );
    world.add_component(unit.cabin_entity, cabin_hitbox);

    // -----   Wheels
    let wheel_config = display::wheel::WheelConfig {
        width: BASE_TRICYCLE_WHEEL_WIDTH,
        radius: BASE_TRICYCLE_WHEEL_RADIUS,
    };

    let wheel = display::wheel::Wheel::from_config(wheel_config);
    let hit_collection =
        components::hit_collection::HitCollection::from_hit_boxes(&wheel.hit_boxes());
    world.add_component(unit.rear_left_wheel_entity, wheel);
    world.add_component(unit.rear_left_wheel_entity, hit_collection);

    let wheel = display::wheel::Wheel::from_config(wheel_config);
    let hit_collection =
        components::hit_collection::HitCollection::from_hit_boxes(&wheel.hit_boxes());
    world.add_component(unit.rear_right_wheel_entity, wheel);
    world.add_component(unit.rear_right_wheel_entity, hit_collection);

    let wheel = display::wheel::Wheel::from_config(wheel_config);
    let hit_collection =
        components::hit_collection::HitCollection::from_hit_boxes(&wheel.hit_boxes());
    world.add_component(unit.front_left_wheel_entity, wheel);
    world.add_component(unit.front_left_wheel_entity, hit_collection);
    world.add_component(
        unit.front_left_wheel_entity,
        display::wheeled_steer_beam::WheeledSteerBeam {},
    );

    let wheel = display::wheel::Wheel::from_config(wheel_config);
    let hit_collection =
        components::hit_collection::HitCollection::from_hit_boxes(&wheel.hit_boxes());
    world.add_component(unit.front_right_wheel_entity, wheel);
    world.add_component(unit.front_right_wheel_entity, hit_collection);

    // -----   Flag
    world.add_component(
        unit.flag_entity,
        display::flag::Flag::from_scale_color(0.5, display::Color::RED),
    );
    world.add_component(
        unit.flag_entity,
        Pose::from_xyz(
            -0.1,
            -display::wheeled_body::WHEELED_BODY_CABIN_WIDTH / 2.0 + 0.05,
            0.0,
        )
        .rotated_angle_z(cgmath::Deg(180.0)),
    );
    world.add_component(unit.flag_entity, Parent::new(unit.cabin_entity));

    // -----   Health Bar
    world.add_component(
        unit.health_bar_entity,
        Pose::from_xyz(
            0.0,
            0.0,
            display::wheeled_body::WHEELED_BODY_CABIN_HEIGHT / 2.0,
        )
        .rotated_angle_z(cgmath::Deg(90.0)),
    );
    world.add_component(
        unit.health_bar_entity,
        display::health_bar::HealthBar::new(unit.unit_entity, 0.6),
    );
    world.add_component(unit.health_bar_entity, Parent::new(unit.cabin_entity));
}
