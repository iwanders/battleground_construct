use super::components;
use crate::display::flag::Flag;
use crate::display::tank_body::TankBody;
use engine::prelude::*;

pub struct TeamColorTank {}
impl System for TeamColorTank {
    fn update(&mut self, world: &mut World) {
        for (entity, team_member) in world.component_iter::<components::team_member::TeamMember>() {
            if let Some(team) = world.component::<components::team::Team>(team_member.team()) {
                // Now that we have the team, we can iterate over the elements:
                if let Some(group) = world.component::<components::group::Group>(entity) {
                    for v in group.entities() {
                        if let Some(mut flag) = world.component_mut::<Flag>(*v) {
                            flag.set_color(*team.color());
                        }
                        if let Some(mut tank) = world.component_mut::<TankBody>(*v) {
                            tank.set_color(*team.color());
                        }
                    }
                }
            }
        }
    }
}
