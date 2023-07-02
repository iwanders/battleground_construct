use super::Unit;
use crate::components;
use crate::display;
use crate::display::primitives::Vec3;
use cgmath::Deg;
use engine::prelude::*;
use serde::{Deserialize, Serialize};

use components::group::Group;
use components::parent::Parent;
use components::pose::{Pose, PreTransform};

pub struct ArmSpawnConfig {
    pub x: f32,
    pub y: f32,
    pub yaw: f32,
    pub controller: Box<dyn battleground_unit_control::UnitControl>,
}

#[derive(Deserialize, Serialize, Debug, Clone, Copy)]
pub struct UnitArm {
    pub unit_entity: EntityId,
    pub control_entity: EntityId,
    pub base_entity: EntityId,
    pub base_revolute_entity: EntityId,
    pub arm_entity: EntityId,
    pub lower_arm_entity: EntityId,
    pub tip_entity: EntityId,
}
impl Component for UnitArm {}

impl Unit for UnitArm {
    fn children(&self) -> Vec<EntityId> {
        vec![
            self.control_entity,
            self.base_entity,
            self.base_revolute_entity,
            self.arm_entity,
            self.lower_arm_entity,
            self.tip_entity,
        ]
    }
}

/// Spawn an arm.
pub fn spawn_arm(world: &mut World, config: ArmSpawnConfig) -> EntityId {
    /*

        Unit Entity:
            - Health
            - TeamMember
            - Eternal

        Control Entity:
            - UnitController

        Base Entity:
            - Revolute
            -> Arm
                - Revolute
                -> Arm...

        The Unit and Control entities are 'free'.
    */
    let unit_entity = world.add_entity();
    let control_entity = world.add_entity();

    let base_entity = world.add_entity();
    let base_revolute_entity = world.add_entity();

    let arm_entity = world.add_entity();
    let lower_arm_entity = world.add_entity();
    let tip_entity = world.add_entity();

    let unit_arm = UnitArm {
        unit_entity,
        control_entity,
        base_entity,
        base_revolute_entity,
        arm_entity,
        lower_arm_entity,
        tip_entity,
    };
    // Unit must be first in the group!
    let mut tank_group_entities: Vec<EntityId> = vec![unit_entity];
    tank_group_entities.append(&mut unit_arm.children());

    // Create the register interface, we'll add modules throughout this function.
    let register_interface = components::unit_interface::RegisterInterfaceContainer::new(
        components::unit_interface::RegisterInterface::new(),
    );
    super::common::add_common_global(&register_interface);

    world.add_component(unit_entity, unit_arm);

    let unit_id = super::common::add_common_unit(
        world,
        &register_interface,
        unit_entity,
        battleground_unit_control::units::UnitType::Unknown,
    );

    add_arm_passive(world, &unit_arm);

    // -----   Base
    world.add_component(base_entity, Pose::from_se2(config.x, config.y, config.yaw));

    // Base revolute.

    world.add_component(
        base_revolute_entity,
        PreTransform::new()
            .rotated_angle_z(Deg(-90.0))
            .rotated_angle_y(Deg(-90.0)),
    );
    let revolute_config = components::revolute::RevoluteConfig {
        axis: Vec3::new(1.0, 0.0, 0.0),
        velocity_bounds: (-1.0, 1.0),
        acceleration_bounds: Some((-1.0, 1.0)),
        velocity_cmd: 0.1,
        ..Default::default()
    };
    super::common::add_revolute(
        world,
        &register_interface,
        base_revolute_entity,
        "base",
        1,
        revolute_config,
    );
    world.add_component(base_revolute_entity, Parent::new(base_entity));

    // -----   arm
    let revolute_config = components::revolute::RevoluteConfig {
        axis: Vec3::new(0.0, 1.0, 0.0),
        velocity_bounds: (-1.0, 1.0),
        acceleration_bounds: Some((-1.0, 1.0)),
        velocity_cmd: 0.3,
        ..Default::default()
    };
    super::common::add_revolute(
        world,
        &register_interface,
        arm_entity,
        "arm",
        2,
        revolute_config,
    );
    world.add_component(
        arm_entity,
        PreTransform::from_translation(Vec3::new(1.0, 0.0, 0.0)),
    );
    world.add_component(arm_entity, Parent::new(base_revolute_entity));

    // -----   lower arm
    let revolute_config = components::revolute::RevoluteConfig {
        axis: Vec3::new(0.0, 1.0, 0.0),
        velocity_bounds: (-1.0, 1.0),
        acceleration_bounds: Some((-1.0, 1.0)),
        velocity_cmd: 0.3,
        ..Default::default()
    };
    super::common::add_revolute(
        world,
        &register_interface,
        lower_arm_entity,
        "lower_arm",
        3,
        revolute_config,
    );
    world.add_component(
        lower_arm_entity,
        PreTransform::from_translation(Vec3::new(1.0, 0.0, 0.0)),
    );
    world.add_component(lower_arm_entity, Parent::new(arm_entity));

    // -----   tip
    world.add_component(
        tip_entity,
        PreTransform::from_translation(Vec3::new(1.0, 0.0, 0.0)),
    );
    world.add_component(tip_entity, Parent::new(lower_arm_entity));
    let particle_effect_id = components::id_generator::generate_id(world);
    world.add_component(
        tip_entity,
        display::particle_emitter::ParticleEmitter::bullet_trail(
            particle_effect_id,
            0.05,
            display::Color::RED,
        ),
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
    let mut tank_group_entities: Vec<EntityId> = vec![unit_entity];
    tank_group_entities.append(&mut unit_arm.children());

    let group = Group::from(&tank_group_entities);
    for e in tank_group_entities.iter() {
        world.add_component(*e, group.clone());
        world.add_component(*e, components::unit_member::UnitMember::new(unit_id));
    }

    unit_entity
}

pub fn add_arm_passive(world: &mut World, unit: &UnitArm) {
    // -----   Body
    let body = display::arm_joint::ArmJoint::new().inline();
    world.add_component(unit.base_revolute_entity, body);
    let segment = display::arm_segment::ArmSegment::new();
    world.add_component(unit.base_revolute_entity, segment);

    // well, this all isn't really elegant.

    let boxes = segment.hit_boxes();
    let hit_collection = components::hit_collection::HitCollection::from_hit_boxes(&boxes);
    // world.add_component(unit.base_revolute_entity, display::debug_hit_collection::DebugHitCollection::from_hit_collection(&hit_collection));
    world.add_component(
        unit.base_revolute_entity,
        components::select_box::SelectBox::from_hit_box(&boxes[0].1),
    );
    world.add_component(unit.base_revolute_entity, hit_collection);

    // -----   Turret
    let arm_joint = display::arm_joint::ArmJoint::new();
    world.add_component(unit.arm_entity, arm_joint);
    let segment = display::arm_segment::ArmSegment::new();
    world.add_component(unit.arm_entity, segment);
    let hit_collection =
        components::hit_collection::HitCollection::from_hit_boxes(&segment.hit_boxes());
    world.add_component(unit.arm_entity, hit_collection);

    let lower_arm_joint = display::arm_joint::ArmJoint::new();
    world.add_component(unit.lower_arm_entity, lower_arm_joint);
    world.add_component(
        unit.lower_arm_entity,
        display::arm_segment::ArmSegment::new(),
    );
}
