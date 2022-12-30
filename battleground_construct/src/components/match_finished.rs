use super::team::TeamId;
use engine::prelude::*;

use super::match_king_of_the_hill::MatchKingOfTheHillReport;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub enum MatchConclusion {
    /// Match was concluded based on a time limit being reached.
    TimeLimit,
    /// Match was concluded based on the objectives criteria being met.
    Criteria,
    // Could have a CriteriaAccelerated here, in case we own the capture points, no no other
    // possible contenders are alive, in that case there's no need to wait around.
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ObjectiveReport {
    MatchKingOfTheHillReport(MatchKingOfTheHillReport),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MatchReport {
    /// The winner of the match, if any. Technically, this ought to be a list... to support matches
    /// ending in a draw, but that just makes things complicated.
    pub winner: Option<TeamId>,
    /// Reports by individual objectives.
    pub reports: Vec<ObjectiveReport>,
    /// Cause of the match finish declaration.
    pub conclusion: MatchConclusion,
    /// Time at which the match was declared finished.
    pub duration: f32,
}

#[derive(Debug, Clone)]
pub struct MatchFinished {
    report: Option<MatchReport>,
}

impl MatchFinished {
    pub fn new() -> Self {
        Self { report: None }
    }

    pub fn from_report(report: MatchReport) -> Self {
        Self {
            report: Some(report),
        }
    }

    pub fn report(&self) -> Option<&MatchReport> {
        self.report.as_ref()
    }
}
impl Component for MatchFinished {}
