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
}
impl Component for UnitArm {}

impl Unit for UnitArm {
    fn children(&self) -> Vec<EntityId> {
        vec![
            self.control_entity,
            self.base_entity,
            self.base_revolute_entity,
            self.arm_entity,
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

    let unit_arm = UnitArm {
        unit_entity,
        control_entity,
        base_entity,
        base_revolute_entity,
        arm_entity,
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
        PreTransform::new().rotated_angle_y(Deg(-90.0)),
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
        "turret",
        1,
        revolute_config,
    );
    world.add_component(base_revolute_entity, Parent::new(base_entity));

    // -----   Turret
    let revolute_config = components::revolute::RevoluteConfig {
        axis: Vec3::new(1.0, 0.0, 0.0),
        velocity_bounds: (-1.0, 1.0),
        acceleration_bounds: Some((-1.0, 1.0)),
        velocity_cmd: 0.3,
        ..Default::default()
    };
    super::common::add_revolute(
        world,
        &register_interface,
        arm_entity,
        "turret",
        1,
        revolute_config,
    );
    world.add_component(
        arm_entity,
        PreTransform::from_translation(Vec3::new(1.0, 0.0, 0.0)).rotated_angle_z(Deg(90.0)),
    );
    world.add_component(arm_entity, Parent::new(base_revolute_entity));

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
    let body = display::arm_joint::ArmJoint::new();
    // let hitbox = body.hitbox();
    world.add_component(unit.base_revolute_entity, body);
    // world.add_component(
    // unit.base_entity,
    // components::select_box::SelectBox::from_hit_box(&hitbox),
    // );
    // world.add_component(unit.base_entity, hitbox);

    // -----   Turret
    let tank_turret = display::arm_joint::ArmJoint::new();
    // let hitbox = tank_turret.hitbox();
    // world.add_component(unit.arm_entity, hitbox);
    // world.add_component(
    // unit.arm_entity,
    // components::select_box::SelectBox::from_hit_box(&hitbox),
    // );
    world.add_component(unit.arm_entity, tank_turret);
}
