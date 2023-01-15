use crate::components;
use components::match_king_of_the_hill::MatchKingOfTheHillReport;
use components::match_team_deathmatch::MatchTeamDeathmatchReport;
use components::team::TeamId;
use engine::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MatchDominationReport {
    points_team_deathmatch: i64,
    team_deathmatch_report: Option<MatchTeamDeathmatchReport>,
    king_of_the_hill_report: Option<MatchKingOfTheHillReport>,
    capturables: Vec<Option<TeamId>>,
}

impl MatchDominationReport {
    pub fn get_leader(&self) -> Option<TeamId> {
        // If we won by king of the hills, return that leader.
        if self
            .king_of_the_hill_report
            .as_ref()
            .map(|t| t.is_finished())
            .unwrap_or(false)
        {
            return self
                .king_of_the_hill_report
                .as_ref()
                .map(|t| t.get_leader())?;
        }
        // We didn't win by king of the hill, we must've won by holding all capturables and
        // exceeding the required points in the death match category.
        *self.capturables.first()?
    }
}

/// This game type is a combindation of team deathmatch and king of the hill. It terminates either
/// when the king of the hill criteria is met, or if the team death match counter exceeds the
/// specified values AND that team holds all the capturables.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MatchDomination {
    points_team_deathmatch: i64,
    team_deathmatch_report: Option<MatchTeamDeathmatchReport>,
    king_of_the_hill_report: Option<MatchKingOfTheHillReport>,
    capturables: Vec<Option<TeamId>>,
}

impl MatchDomination {
    pub fn new(points_team_deathmatch: i64) -> Self {
        Self {
            points_team_deathmatch,
            team_deathmatch_report: None,
            king_of_the_hill_report: None,
            capturables: vec![],
        }
    }

    pub fn set_team_deathmath_report(&mut self, report: MatchTeamDeathmatchReport) {
        self.team_deathmatch_report = Some(report);
    }

    pub fn set_king_of_the_hill_report(&mut self, report: MatchKingOfTheHillReport) {
        self.king_of_the_hill_report = Some(report);
    }

    pub fn set_capturables(&mut self, capturables: &[Option<TeamId>]) {
        self.capturables = capturables.to_vec();
    }

    pub fn is_finished(&self) -> bool {
        // Check if we win by king of the hill, this is instant victory.
        if self
            .king_of_the_hill_report
            .as_ref()
            .map(|r| r.is_finished())
            .unwrap_or(false)
        {
            return true;
        }

        // King of the hill didn't win yet. Check if we have a death match report, if it exceeds
        // the number of points we need, check if that team owns all the capturables.
        if let Some(report) = &self.team_deathmatch_report {
            for (team_id, team_score) in report.points() {
                if team_score >= self.points_team_deathmatch {
                    if self
                        .capturables
                        .iter()
                        .all(|v| v.map(|t| team_id == t).unwrap_or(false))
                    {
                        // team exceeds required score AND holds all the capturables, the match is
                        // over.
                        return true;
                    }
                }
            }
        }

        false
    }

    pub fn report(&self) -> MatchDominationReport {
        MatchDominationReport {
            points_team_deathmatch: self.points_team_deathmatch,
            team_deathmatch_report: self.team_deathmatch_report.clone(),
            king_of_the_hill_report: self.king_of_the_hill_report.clone(),
            capturables: self.capturables.clone(),
        }
    }
}
impl Component for MatchDomination {}
