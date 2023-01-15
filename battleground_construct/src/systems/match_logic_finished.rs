use crate::components;
use crate::components::team::TeamId;
use components::match_domination::MatchDomination;
use components::match_finished::{MatchConclusion, MatchFinished, MatchReport, ObjectiveReport};
use components::match_king_of_the_hill::MatchKingOfTheHill;
use components::match_team_deathmatch::MatchTeamDeathmatch;
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
                conclusion = Some(MatchConclusion::Objective);
                break;
            }
        }
        // Check death match
        for (_e, match_team_deathmatch) in world.component_iter::<MatchDomination>() {
            if match_team_deathmatch.is_finished() {
                is_finished = true;
                conclusion = Some(MatchConclusion::Objective);
                break;
            }
        }

        for (_e, match_domination) in world.component_iter::<MatchTeamDeathmatch>() {
            if match_domination.is_finished() {
                is_finished = true;
                conclusion = Some(MatchConclusion::Objective);
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
            let mut winners: std::collections::HashSet<TeamId> = Default::default();
            let mut leaders: std::collections::HashSet<TeamId> = Default::default();
            {
                for (_e, match_koth) in world.component_iter::<MatchKingOfTheHill>() {
                    let report = match_koth.clone();
                    if let Some(leader) = report.get_leader() {
                        if report.is_finished() {
                            winners.insert(leader);
                        } else {
                            leaders.insert(leader);
                        }
                    }
                    reports.push(ObjectiveReport::MatchKingOfTheHill(report));
                }
                for (_e, match_team_deathmatch) in world.component_iter::<MatchTeamDeathmatch>() {
                    let report = match_team_deathmatch.clone();
                    if let Some(leader) = report.get_leader() {
                        if report.is_finished() {
                            winners.insert(leader.0);
                        } else {
                            leaders.insert(leader.0);
                        }
                    }
                    reports.push(ObjectiveReport::MatchTeamDeathmatch(report));
                }
                for (_e, match_domination) in world.component_iter::<MatchDomination>() {
                    let report = match_domination.clone();
                    if let Some(leader) = report.get_leader() {
                        if report.is_finished() {
                            winners.insert(leader);
                        } else {
                            leaders.insert(leader);
                        }
                    }
                    reports.push(ObjectiveReport::MatchDomination(report));
                }
            }

            // We are actually finished... lets collect the information for the match report.
            if winners.len() > 1 {
                println!("Got multiple winners: {winners:?}, logic error or draw??");
            }

            // Lets... get the winner.
            let mut final_winner: Option<TeamId> = None;

            // If there is a true winner...
            if !winners.is_empty() {
                final_winner = winners.iter().copied().collect::<Vec<_>>().first().copied();
            }

            // Check if we don't have a winner, if not, it is decided by time limit, use the runner
            // up.
            if final_winner.is_none() {
                if leaders.len() > 1 {
                    println!(
                        "Got multiple runner ups: {leaders:?}... logic error? Draw? Taking first."
                    );
                }
                final_winner = leaders.iter().copied().collect::<Vec<_>>().first().copied();
            }

            // Now, we can create the match report.
            let report = MatchReport {
                winner: final_winner,
                conclusion: conclusion.unwrap(),
                reports,
                duration,
            };
            // println!("Match finished: {report:#?}");
            let id = world.add_entity();
            world.add_component(id, MatchFinished::from_report(report));

            // Spawn the victory effect update tracker.
            let victory_effect_id = world.add_entity();
            world.add_component(
                victory_effect_id,
                components::victory_effect::VictoryEffect::default(),
            );
        }
    }
}
