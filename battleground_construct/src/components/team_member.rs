use engine::prelude::*;

use crate::components::team::TeamId;

#[derive(Debug, Clone, Copy)]
pub struct TeamMember {
    team: TeamId,
}

impl TeamMember {
    pub fn new(team: TeamId) -> Self {
        TeamMember { team }
    }
    pub fn team(&self) -> TeamId {
        self.team
    }
}
impl Component for TeamMember {}
