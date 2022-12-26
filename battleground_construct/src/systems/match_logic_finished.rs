use crate::components;
use components::match_finished::MatchFinished;
use components::match_king_of_the_hill::MatchKingOfTheHill;
use components::match_time_limit::MatchTimeLimit;

use engine::prelude::*;

pub struct MatchLogicFinished {}
impl System for MatchLogicFinished {
    fn update(&mut self, world: &mut World) {
        if let Some(_) = world.component_iter::<MatchFinished>().next() {
            return; // match is already finished.
        }

        let mut is_finished = false;

        // Check king of the hill criteria.
        for (_e, match_koth) in world.component_iter::<MatchKingOfTheHill>() {
            if match_koth.is_finished() {
                is_finished = true;
                break;
            }
        }

        // Check time limit criteria.
        for (_e, match_time_limit) in world.component_iter::<MatchTimeLimit>() {
            // println!("Match match_time_limit: {:?}", match_time_limit);
            if match_time_limit.is_finished() {
                is_finished = true;
                // println!("match_time_limit decides finished");
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
