use super::components;
use crate::display::tank_body::TankBody;
use engine::prelude::*;

pub struct TeamColorTankBody {}
impl System for TeamColorTankBody {
    fn update(&mut self, world: &mut World) {
        for (entity, team_member) in world.component_iter::<components::team_member::TeamMember>() {
            if let Some(team) = world.component::<components::team::Team>(team_member.team()) {
                // try to see if we can find a velocity for this entity.
                if let Some(mut tank) = world.component_mut::<TankBody>(entity) {
                    tank.set_color(*team.color());
                }
            }
        }
    }
}
