use engine::prelude::*;

#[derive(Debug, Clone, Copy)]
pub struct TeamMember {
    team: EntityId,
}

impl TeamMember {
    pub fn new(team: EntityId) -> Self {
        TeamMember { team }
    }
    pub fn team(&self) -> EntityId {
        self.team
    }
}
impl Component for TeamMember {}
