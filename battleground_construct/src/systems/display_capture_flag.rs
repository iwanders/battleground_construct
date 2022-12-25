use super::components;
use super::display;
use crate::components::team::{get_team_entity, Team};
use components::capturable::Capturable;
use display::display_control_point::DisplayControlPoint;
use display::flag::Flag;

use engine::prelude::*;

pub struct DisplayCaptureFlag {}
impl System for DisplayCaptureFlag {
    fn update(&mut self, world: &mut World) {
        for (capturable_entity, capturable) in world.component_iter::<Capturable>() {
            // Obtain the current color that holds the capturable.
            let color = capturable
                .owner()
                .and_then(|owner| get_team_entity(world, owner))
                .and_then(|owner_entity| world.component::<Team>(owner_entity))
                .map_or(display::Color::GREY, |team| *team.color());

            // If there is a flag here, modify it.
            if let Some(mut flag) = world.component_mut::<Flag>(capturable_entity) {
                flag.set_color(color);
                flag.set_flag_position(capturable.strength());
            };

            // If there is an displayable control point here, color it appropriately.
            if let Some(mut area) = world.component_mut::<DisplayControlPoint>(capturable_entity) {
                area.set_color(color);
                if let Some(point) =
                    world.component::<components::capture_point::CapturePoint>(capturable_entity)
                {
                    area.set_radius(point.radius());
                }
            }
        }
    }
}
