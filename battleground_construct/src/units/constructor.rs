use super::base_tricycle::{spawn_base_tricycle, BaseTricycle, BaseTricycleSpawnConfig};
use super::common::{add_component_box, ComponentBox, ComponentBoxSpawnConfig};
use super::{Unit, UnitId};
use crate::components;
// use crate::display;
use crate::display::primitives::Vec3;
// use components::group::Group;
use components::parent::Parent;
use components::pose::PreTransform;
use engine::prelude::*;
use serde::{Deserialize, Serialize};

// use battleground_unit_control::units::constructor::*;

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
    pub base: BaseTricycle,
    pub right_box: ComponentBox,
    pub left_box: ComponentBox,
}
impl Component for UnitConstructor {}

impl Unit for UnitConstructor {
    fn children(&self) -> Vec<EntityId> {
        let mut r = self.base.children();
        r.push(self.right_box.base);
        r.push(self.right_box.lid);
        r.push(self.left_box.base);
        r.push(self.left_box.lid);
        r
    }
    fn unit_entity(&self) -> EntityId {
        self.base.unit_entity()
    }
    fn unit_id(&self) -> UnitId {
        self.base.unit_id()
    }
}

/// Spawn a constructor, returning the unit entity.
pub fn spawn_constructor(world: &mut World, config: ConstructorSpawnConfig) -> EntityId {
    let base_config = BaseTricycleSpawnConfig {
        x: config.x,
        y: config.y,
        yaw: config.yaw,
        controller: config.controller,
        team_member: config.team_member,
        radio_config: config.radio_config,
    };

    // Payload size is 1.5 long, 1 wide.
    // Payload protrudes 0.5 behind base link. Payload geometric center at (0.75, 0)

    let base = spawn_base_tricycle(
        world,
        base_config,
        battleground_unit_control::units::UnitType::Constructor,
    );

    let box_config = ComponentBoxSpawnConfig {
        length: 1.25,
        height: 0.2,
        width: 0.4,
    };
    let right_box = add_component_box(world, box_config);

    world.add_component(right_box.base, Parent::new(base.payload_entity));
    world.add_component(
        right_box.base,
        PreTransform::from_translation(Vec3::new(0.0, -0.25, 0.0)),
    );

    let left_box = add_component_box(world, box_config);

    world.add_component(left_box.base, Parent::new(base.payload_entity));
    world.add_component(
        left_box.base,
        PreTransform::from_translation(Vec3::new(0.0, 0.25, 0.0))
            .rotated_angle_z(cgmath::Deg(180.0)),
    );

    let unit_constructor = UnitConstructor {
        base,
        right_box,
        left_box,
    };

    let register_interface = super::common::get_register_interface(world, base.control_entity);

    let deploy_config = components::deploy::DeployConfig {
        deploy_function: std::rc::Rc::new(deploy_function),
    };

    super::common::add_common_deploy(
        world,
        &register_interface,
        base.control_entity,
        deploy_config,
    );
    world
        .component_mut::<components::deploy::Deploy>(base.control_entity)
        .unwrap()
        .set_desired_state(components::deploy::DeployState::Deployed);

    super::common::add_group_team_unit(world, &unit_constructor, config.team_member);
    world.add_component(unit_constructor.unit_entity(), unit_constructor);

    base.unit_entity
}

pub fn deploy_function(world: &mut World, control_entity: EntityId) {
    let (desired, state) = {
        let deploy = world.component_mut::<components::deploy::Deploy>(control_entity);
        if deploy.is_none() {
            return;
        }
        let deploy = deploy.unwrap();

        let desired = deploy.get_desired_state();
        let state = deploy.get_state();
        (desired, state)
    };

    if desired == state {
        return;
    }

    // get the unit entity.
    let constructor = {
        let unit_entity = components::unit::get_unit_entity_of(world, control_entity);
        if unit_entity.is_none() {
            return;
        }
        let unit_entity = unit_entity.unwrap();

        let constructor = world.component::<UnitConstructor>(unit_entity);
        if constructor.is_none() {
            return;
        }
        *constructor.unwrap()
    };

    let current_state;

    if desired == components::deploy::DeployState::Deployed {
        let left_state = constructor
            .left_box
            .deploy(world, components::deploy::DeployState::Deployed);
        let right_state = constructor
            .right_box
            .deploy(world, components::deploy::DeployState::Deployed);
        if left_state == desired && right_state == desired {
            current_state = components::deploy::DeployState::Deployed;
        } else {
            current_state = components::deploy::DeployState::InTransition;
        }
    } else {
        // Desired is normal.
        let left_state = constructor
            .left_box
            .deploy(world, components::deploy::DeployState::Normal);
        let right_state = constructor
            .right_box
            .deploy(world, components::deploy::DeployState::Normal);
        if left_state == desired && right_state == desired {
            current_state = components::deploy::DeployState::Normal;
        } else {
            current_state = components::deploy::DeployState::InTransition;
        }
    }

    let deploy = world.component_mut::<components::deploy::Deploy>(control_entity);
    if deploy.is_none() {
        return;
    }
    let mut deploy = deploy.unwrap();

    deploy.set_state(current_state);
}
