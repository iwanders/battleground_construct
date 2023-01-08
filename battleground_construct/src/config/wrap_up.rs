use super::specification;
use crate::components;
use crate::components::team::TeamId;
use crate::Construct;
use components::match_finished::MatchReport;
use engine::*;
use serde::{Deserialize, Serialize};

// This struct is more elaborate than the struct in match finished. That just contains entity ids
// doesn't consume the mapping back to the actual team names. But it is all that is needed for the
// game logic, so this here is the externally-readable output that contains everything an outside
// system would need to know.

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WrapUpReport {
    pub winning_team: Option<specification::Team>,
    pub match_report: Option<MatchReport>,
    pub teams: std::collections::HashMap<TeamId, specification::Team>,
}

/// Should only be called if MatchFinished is present.
pub fn create_wrap_up_report(world: &World) -> WrapUpReport {
    // First, build the report.
    let match_report = world
        .component_iter::<components::match_finished::MatchFinished>()
        .next()
        .and_then(|(_e, m)| m.report().cloned());

    // Collect all teams.
    let mut teams = std::collections::HashMap::<TeamId, specification::Team>::new();
    {
        for (_e, team) in world.component_iter::<components::team::Team>() {
            let team_color = team.color();
            teams.insert(
                team.id(),
                specification::Team {
                    name: team.name().to_owned(),
                    color: (team_color.r, team_color.g, team_color.b),
                },
            );
        }
    }

    // determine the winning taem.
    let winning_team = match_report.as_ref().and_then(|rp| {
        rp.winner
            .map(|t| teams.get(&t).expect("team must exist").clone())
    });

    // Cool, now we can construct the wrap up report.
    WrapUpReport {
        winning_team,
        match_report,
        teams,
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FullMatchReport {
    pub wrap_up: WrapUpReport,
    pub scenario: specification::ScenarioConfig,
}

pub fn wrap_up_scenario(
    wrap_up: super::specification::WrapUpConfig,
    construct: &Construct,
) -> Result<FullMatchReport, Box<dyn std::error::Error>> {
    let full_report = FullMatchReport {
        wrap_up: create_wrap_up_report(construct.world()),
        scenario: wrap_up.scenario,
    };

    // Now, check if according to the scenario we have to do anything with the report, like writing it.
    if let Some(path) = wrap_up.write_wrap_up {
        use std::io::Write;
        let mut file = std::fs::File::create(path)?;
        file.write_all(serde_yaml::to_string(&full_report)?.as_bytes())?;
    }

    Ok(full_report)
}
