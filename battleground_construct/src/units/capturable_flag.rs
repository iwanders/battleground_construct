use crate::components;
use crate::display;
use components::pose::Pose;
use engine::prelude::*;

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

pub fn spawn_capturable_flag(world: &mut World, config: CapturableFlagConfig) -> EntityId {
    let capturable_flag = world.add_entity();

    world.add_component(
        capturable_flag,
        Pose::from_se2(config.x, config.y, config.yaw),
    );
    let capturable = components::capturable::Capturable::new(
        config.initial_owner,
        config.capture_strength,
        config.capture_type,
    );

    world.add_component(capturable_flag, capturable);
    world.add_component(
        capturable_flag,
        components::capture_point::CapturePoint::new(config.radius, config.capture_speed),
    );
    let mut flag = display::flag::Flag::new();
    flag.set_pole_height(2.0);
    world.add_component(capturable_flag, flag);

    world.add_component(
        capturable_flag,
        display::display_control_point::DisplayControlPoint::new(),
    );

    capturable_flag
}
