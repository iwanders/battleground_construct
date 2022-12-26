use crate::components;
use components::match_finished::MatchFinished;
use components::match_king_of_the_hill::MatchKingOfTheHill;

use engine::prelude::*;

pub struct MatchLogicFinished {}
impl System for MatchLogicFinished {
    fn update(&mut self, world: &mut World) {
        if let Some(_) = world.component_iter::<MatchFinished>().next() {
            return; // match is already finished.
        }

        let mut is_finished = false;
        for (_e, match_koth) in world.component_iter::<MatchKingOfTheHill>() {
            // println!("Match koth: {:?}", match_koth);
            if match_koth.is_finished() {
                is_finished = true;
                // println!("King of the hill decides finished");
                break;
            }
        }

        if is_finished {
            // add the marker
            let id = world.add_entity();
            world.add_component(id, MatchFinished::new());
        }
    }
}
