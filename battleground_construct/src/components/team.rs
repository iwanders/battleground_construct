use crate::display;
use engine::prelude::*;

#[derive(Copy, Clone, Debug, Eq, Ord, PartialOrd, PartialEq)]
pub struct TeamId(u64);

#[derive(Debug, Clone)]
pub struct Team {
    id: TeamId,
    name: String,
    color: display::Color,
}

impl Team {
    pub fn new(id: u64, name: &str, color: display::Color) -> Self {
        Team {
            name: name.to_owned(),
            id: TeamId(id),
            color,
        }
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn color(&self) -> &display::Color {
        &self.color
    }
    pub fn id(&self) -> TeamId {
        self.id
    }
}
impl Component for Team {}

pub fn get_team_entity(world: &World, team_id: TeamId) -> Option<EntityId> {
    for (entity, v) in world.component_iter::<Team>() {
        if v.id() == team_id {
            return Some(entity);
        }
    }
    None
}
