use crate::components;
use components::capturable::Capturable;
use components::match_domination::MatchDomination;
use components::match_king_of_the_hill::MatchKingOfTheHill;
use components::match_team_deathmatch::MatchTeamDeathmatch;
use components::team::TeamId;

use engine::prelude::*;

pub struct MatchLogicDomination {}
impl System for MatchLogicDomination {
    fn update(&mut self, world: &mut World) {
        let mut owners: Vec<Option<TeamId>> = Default::default();
        for (_e, capturable) in world.component_iter::<Capturable>() {
            owners.push(capturable.owner());
        }

        let team_deathmatch_report = world
            .component_iter::<MatchTeamDeathmatch>()
            .next()
            .map(|t| t.1.report());
        if team_deathmatch_report.is_none() {
            return;
        }
        let team_deathmatch_report = team_deathmatch_report.unwrap();

        let koth_report = world
            .component_iter::<MatchKingOfTheHill>()
            .next()
            .map(|t| t.1.report());
        if koth_report.is_none() {
            return;
        }
        let koth_report = koth_report.unwrap();

        {
            if let Some((_e, ref mut domination)) =
                world.component_iter_mut::<MatchDomination>().next()
            {
                domination.set_team_deathmath_report(team_deathmatch_report);
                domination.set_king_of_the_hill_report(koth_report);
                domination.set_capturables(&owners);
            }
        }
    }
}
