use super::specification;
use crate::components;
use crate::components::team::TeamId;
use crate::Construct;
use components::match_finished::MatchReport;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WrapUpReport {
    pub winning_team: Option<specification::Team>,
    pub match_report: Option<MatchReport>,
    pub teams: std::collections::HashMap<TeamId, specification::Team>,
    pub scenario: specification::ScenarioConfig,
}

pub fn wrap_up_scenario(
    wrap_up: super::specification::WrapUpConfig,
    construct: &Construct,
) -> Result<WrapUpReport, Box<dyn std::error::Error>> {
    // First, build the report.
    let match_report = construct
        .world()
        .component_iter::<components::match_finished::MatchFinished>()
        .next()
        .map(|(_e, m)| m.report().map(|z| z.clone()))
        .flatten();

    // Collect all teams.
    let mut teams = std::collections::HashMap::<TeamId, specification::Team>::new();
    {
        for (_e, team) in construct.world().component_iter::<components::team::Team>() {
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
    let winning_team = match_report
        .as_ref()
        .map(|rp| {
            rp.winner
                .map(|t| teams.get(&t).expect("team must exist").clone())
        })
        .flatten();

    // Cool, now we can construct the wrap up report.
    let wrap_up_report = WrapUpReport {
        winning_team: winning_team,
        match_report: match_report,
        teams,
        scenario: wrap_up.scenario,
    };

    // Now, check if according to the scenario we have to do anything with the report, like writing it.
    if let Some(path) = wrap_up.write_wrap_up {
        use std::io::Write;
        let mut file = std::fs::File::create(path)?;
        file.write_all(serde_yaml::to_string(&wrap_up_report)?.as_bytes())?;
    }

    Ok(wrap_up_report)
}
