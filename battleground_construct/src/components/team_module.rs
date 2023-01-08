use crate::components;
use engine::prelude::*;

use crate::components::unit_interface::{Register, RegisterMap, UnitModule};

#[derive(Debug, Clone, Copy)]
pub struct TeamModule {
    entity: EntityId,
}

impl TeamModule {
    pub fn new(entity: EntityId) -> Self {
        TeamModule { entity }
    }
}
impl Component for TeamModule {}

impl UnitModule for TeamModule {
    fn get_registers(&self, world: &World, registers: &mut RegisterMap) {
        // get the team membership of this entity.
        let value = world
            .component::<components::team_member::TeamMember>(self.entity)
            .map(|t| t.team().as_u64() as i32)
            .unwrap_or(battleground_unit_control::modules::team::TEAM_NO_TEAM);
        registers.insert(
            battleground_unit_control::modules::team::REG_TEAM_TEAMID,
            Register::new_i32("team", value),
        );
    }
}
