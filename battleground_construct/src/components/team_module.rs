use engine::prelude::*;
use crate::components;

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
        let value = world.component::<components::team_member::TeamMember>(self.entity).and_then(|t| Some(t.team().as_u64() as i32)).unwrap_or(-1);
        registers.insert(
            battleground_unit_control::modules::team::register::TEAM, Register::new_i32("team", value),
        );
    }
}
