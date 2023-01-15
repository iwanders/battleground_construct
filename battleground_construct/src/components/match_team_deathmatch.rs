use crate::components;
use components::team::TeamId;
use engine::prelude::*;
use serde::{Deserialize, Serialize};

/// Marker component added whenever something is destroyed, for the death match logic to clean up
/// and count towards the score.
#[derive(Debug, Clone, Copy)]
pub struct MatchTeamDeathmatchJustDestroyed;
impl Component for MatchTeamDeathmatchJustDestroyed {}



#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MatchTeamDeathmatch {
    points: std::collections::HashMap<TeamId, i64>,
    point_limit: Option<i64>,
}

impl MatchTeamDeathmatch {
    pub fn new(point_limit: Option<i64>) -> Self {
        Self {
            point_limit,
            points: Default::default(),
        }
    }


    pub fn get_leader(&self) -> Option<(TeamId, i64)> {
        self.points
            .iter()
            .max_by(|a, b| a.1.cmp(b.1))
            .map(|t| (*t.0, *t.1))
    }

    pub fn point_limit(&self) -> Option<i64> {
        self.point_limit
    }

    pub fn points(&self) -> Vec<(TeamId, i64)> {
        let mut v: Vec<(TeamId, i64)> = self.points.iter().map(|(t, s)| (*t, *s)).collect();
        v.sort_by(|a, b| a.0.cmp(&b.0));
        v
    }

    pub fn add_points(&mut self, point_additions: &[(TeamId, i64)]) {
        for (team, value) in point_additions.iter() {
            *self.points.entry(*team).or_insert(0) += value;
        }
    }

    pub fn is_finished(&self) -> bool {
        if let Some(limit) = self.point_limit {
            for (_t, v) in self.points.iter() {
                if *v >= limit {
                    return true;
                }
            }
        }
        false
    }
}
impl Component for MatchTeamDeathmatch {}
