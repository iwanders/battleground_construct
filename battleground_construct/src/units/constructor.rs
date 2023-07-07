use super::base_tricycle::{spawn_base_tricycle, BaseTricycle, BaseTricycleSpawnConfig};
use super::{Unit, UnitId};
use crate::components;
// use crate::display;
// use crate::display::primitives::Vec3;
// use components::group::Group;
// use components::parent::Parent;
// use components::pose::{Pose, PreTransform};
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
    pub thing: EntityId,
}
impl Component for UnitConstructor {}

impl Unit for UnitConstructor {
    fn children(&self) -> Vec<EntityId> {
        let mut r = self.base.children();
        r.push(self.thing);
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

    let base = spawn_base_tricycle(
        world,
        base_config,
        battleground_unit_control::units::UnitType::Constructor,
    );

    let new_entity = world.add_entity();
    let unit_constructor = UnitConstructor {
        base,
        thing: new_entity,
    };

    super::common::add_group_team_unit(world, &unit_constructor, config.team_member);

    base.unit_entity
}
