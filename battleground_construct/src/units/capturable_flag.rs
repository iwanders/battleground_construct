use crate::components;
use crate::display;
use components::pose::Pose;
use engine::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug)]
pub struct CapturableFlagConfig {
    pub x: f32,
    pub y: f32,
    pub yaw: f32,
    pub radius: f32,
    pub capture_speed: f32,
    pub initial_owner: Option<components::team::TeamId>,
    pub capture_strength: f32,
    pub capture_type: components::capturable::CaptureType,
}

impl Default for CapturableFlagConfig {
    fn default() -> Self {
        CapturableFlagConfig {
            x: 0.0,
            y: 0.0,
            yaw: 0.0,
            radius: 1.0,
            capture_speed: 1.0,
            capture_strength: 0.5,
            initial_owner: None,
            capture_type: components::capturable::CaptureType::Exclusive,
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, Copy)]
pub struct UnitCapturableFlag {
    pub capturable_entity: EntityId,
}
impl Component for UnitCapturableFlag {}

pub fn spawn_capturable_flag(world: &mut World, config: CapturableFlagConfig) -> EntityId {
    let capturable_entity = world.add_entity();

    world.add_component(
        capturable_entity,
        Pose::from_se2(config.x, config.y, config.yaw),
    );

    let unit_capturable = UnitCapturableFlag { capturable_entity };
    add_capturable_passives(world, &unit_capturable);
    world.add_component(capturable_entity, unit_capturable);

    world.add_component(
        capturable_entity,
        components::capture_point::CapturePoint::new(config.radius, config.capture_speed),
    );

    let capturable = components::capturable::Capturable::new(
        config.initial_owner,
        config.capture_strength,
        config.capture_type,
    );
    world.add_component(capturable_entity, capturable);

    capturable_entity
}

pub fn add_capturable_passives(world: &mut World, capturable: &UnitCapturableFlag) {
    let mut flag = display::flag::Flag::new();
    flag.set_pole_height(2.0);
    world.add_component(capturable.capturable_entity, flag);

    world.add_component(
        capturable.capturable_entity,
        display::display_control_point::DisplayControlPoint::new(),
    );
}
