use super::components;
use crate::display::flag::Flag;
use crate::display::tank_body::TankBody;
use crate::display::artillery_body::ArtilleryBody;
use engine::prelude::*;

pub struct TeamColorBody {}
impl System for TeamColorBody {
    fn update(&mut self, world: &mut World) {
        for (entity, team_member) in world.component_iter::<components::team_member::TeamMember>() {
            if let Some(team_entity) = components::team::get_team_entity(world, team_member.team())
            {
                let team = world
                    .component::<components::team::Team>(team_entity)
                    .unwrap();
                // Now that we have the team, we can iterate over the elements:
                if let Some(tank) = world.component::<crate::units::tank::UnitTank>(entity) {
                    if let Some(mut flag) = world.component_mut::<Flag>(tank.flag_entity) {
                        flag.set_color(*team.color());
                    }
                    if let Some(mut tank) = world.component_mut::<TankBody>(tank.body_entity) {
                        tank.set_color(*team.color());
                    }
                }
                if let Some(artillery) = world.component::<crate::units::artillery::UnitArtillery>(entity) {
                    if let Some(mut flag) = world.component_mut::<Flag>(artillery.flag_entity) {
                        flag.set_color(*team.color());
                    }
                    if let Some(mut artillery) = world.component_mut::<ArtilleryBody>(artillery.body_entity) {
                        artillery.set_color(*team.color());
                    }
                }
            }
        }
    }
}
