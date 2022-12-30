use crate::components;
use components::team::TeamId;
use engine::prelude::*;

#[derive(Debug, Clone)]
pub struct MatchKingOfTheHill {
    points: std::collections::HashMap<TeamId, f32>,
    point_limit: Option<f32>,
}

impl MatchKingOfTheHill {
    pub fn new(point_limit: Option<f32>) -> Self {
        Self {
            point_limit,
            points: Default::default(),
        }
    }

    pub fn add_points(&mut self, point_additions: &[(TeamId, f32)]) {
        for (team, value) in point_additions.iter() {
            *self.points.entry(*team).or_insert(0.0) += value;
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

    pub fn get_leader(&self) -> Option<TeamId> {
        self.points
            .iter()
            .max_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(k, _v)| k)
            .copied()
    }
}
impl Component for MatchKingOfTheHill {}
