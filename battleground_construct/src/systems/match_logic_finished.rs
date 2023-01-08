use crate::components;
// use crate::components::team::TeamId;
use components::match_finished::{MatchConclusion, MatchFinished, MatchReport, ObjectiveReport};
use components::match_king_of_the_hill::MatchKingOfTheHill;
use components::match_time_limit::MatchTimeLimit;

use engine::prelude::*;

pub struct MatchLogicFinished {}
impl System for MatchLogicFinished {
    fn update(&mut self, world: &mut World) {
        if world.component_iter::<MatchFinished>().next().is_some() {
            return; // match is already finished.
        }

        let mut is_finished = false;
        let mut conclusion = None;

        // Check king of the hill criteria.
        for (_e, match_koth) in world.component_iter::<MatchKingOfTheHill>() {
            if match_koth.is_finished() {
                is_finished = true;
                conclusion = Some(MatchConclusion::Criteria);
                break;
            }
        }

        // Check time limit criteria.
        for (_e, match_time_limit) in world.component_iter::<MatchTimeLimit>() {
            if match_time_limit.is_finished() {
                is_finished = true;
                conclusion = Some(MatchConclusion::TimeLimit);
                break;
            }
        }

        if is_finished {
            let duration = world
                .component_iter::<components::clock::Clock>()
                .next()
                .expect("Should have one clock")
                .1
                .elapsed_as_f32();

            // collect the reports.
            let mut reports = vec![];
            let mut leaders = vec![];
            {
                for (_e, match_koth) in world.component_iter::<MatchKingOfTheHill>() {
                    let report = match_koth.report();
                    leaders.push(report.get_leader());
                    reports.push(ObjectiveReport::MatchKingOfTheHillReport(report));
                }
            }

            // We are actually finished... lets collect the information for the match report.
            if leaders.len() > 1 {
                println!("Got multiple leaders: {leaders:?}, logic error or draw??");
            }
            // Now, we can create the match report.
            let report = MatchReport {
                winner: leaders.first().copied().flatten(),
                conclusion: conclusion.unwrap(),
                reports,
                duration,
            };
            // println!("Match finished: {report:#?}");
            let id = world.add_entity();
            world.add_component(id, MatchFinished::from_report(report));
        }
    }
}
