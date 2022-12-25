use super::components;
use super::display;
use crate::components::team::{get_team_entity, Team};
use components::capturable::Capturable;
use display::flag::Flag;

use engine::prelude::*;

pub struct DisplayCaptureFlag {}
impl System for DisplayCaptureFlag {
    fn update(&mut self, world: &mut World) {
        for (capturable_entity, capturable) in world.component_iter::<Capturable>() {
            // There's got to be a nice .and_then, but it's late...
            if let Some(mut flag) = world.component_mut::<Flag>(capturable_entity) {
                if let Some(owner) = capturable.owner() {
                    let team_entity = get_team_entity(world, owner);
                    if let Some(actual_owner) = team_entity {
                        if let Some(team) = world.component::<Team>(actual_owner) {
                            flag.set_color(*team.color());
                        } else {
                            flag.set_color(display::Color::GREY);
                        };
                    } else {
                        flag.set_color(display::Color::GREY);
                    };
                } else {
                    flag.set_color(display::Color::GREY);
                };
                flag.set_flag_position(capturable.strength());
            }
        }
    }
}
