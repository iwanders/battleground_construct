use crate::components;
use components::match_team_deathmatch::{MatchTeamDeathmatch, MatchTeamDeathmatchJustDestroyed};
use components::team::TeamId;

use engine::prelude::*;

pub struct MatchLogicTeamDeathmatch {}
impl System for MatchLogicTeamDeathmatch {
    fn update(&mut self, world: &mut World) {
        let to_count = world.component_entities::<MatchTeamDeathmatchJustDestroyed>();

        // get the team that landed the finishing blow.
        let mut new_frags: std::collections::HashMap<TeamId, i64> = Default::default();

        for entity in to_count.iter() {
            // There ought to be a hit history on this component.
            let this_entity_team = world
                .component::<components::team_member::TeamMember>(*entity)
                .map(|x| x.team());
            if let Some(history) = world.component::<components::hit_by::HitByHistory>(*entity) {
                // And a last hit.
                if let Some(last_hit) = history.last() {
                    // and the last hit should have a unit id.
                    if let Some(unit_source) = last_hit.source() {
                        // and there should be a unit entity for that unit id.
                        if let Some(unit_entity) =
                            components::unit::get_unit_entity(world, unit_source)
                        {
                            // and that unit entity should have a team member component.
                            if let Some(team_member) =
                                world.component::<components::team_member::TeamMember>(unit_entity)
                            {
                                // Friendly fire, all shooters subtract one kill.
                                if Some(team_member.team()) == this_entity_team {
                                    *new_frags.entry(team_member.team()).or_insert(0) -= 1;
                                } else {
                                    *new_frags.entry(team_member.team()).or_insert(0) += 1;
                                }
                            }
                        }
                    }
                }
            }
        }

        // Finally, update all team death match trackers.
        if let Some((_e, mut deathmatch)) = world.component_iter_mut::<MatchTeamDeathmatch>().next()
        {
            let update_pairs = new_frags
                .iter()
                .map(|(t, v)| (*t, *v))
                .collect::<Vec<(TeamId, i64)>>();
            deathmatch.add_points(&update_pairs);
        }

        // And remove the justdestroyed markers.
        world.remove_components::<MatchTeamDeathmatchJustDestroyed>(&to_count);
    }
}
