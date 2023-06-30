use crate::components;
use crate::components::team::Team;
use crate::units;
use battleground_unit_control::units::UnitType;
use components::match_finished::MatchFinished;

use engine::prelude::*;

pub struct VictoryEffect {}
impl System for VictoryEffect {
    fn update(&mut self, world: &mut World) {
        let report = world
            .component_iter::<MatchFinished>()
            .next()
            .and_then(|x| x.1.report().cloned());
        if report.is_none() {
            return;
        }
        let report = report.unwrap();

        let t = world
            .component_iter::<components::clock::Clock>()
            .next()
            .expect("Should have one clock")
            .1
            .elapsed_as_f32();

        if !world
            .component_iter_mut::<components::victory_effect::VictoryEffect>()
            .next()
            .map(|mut v| v.1.update(t))
            .unwrap_or(false)
        {
            return; // no update right now.
        }

        // We got here, we ought to spawn victory effects!
        if report.winner.is_none() {
            return;
        }
        let winner_team_id = report.winner.unwrap();
        let team_entity = components::team::get_team_entity(world, winner_team_id);
        if team_entity.is_none() {
            return;
        }
        let team_entity = team_entity.unwrap();

        let color = world.component::<Team>(team_entity).map(|x| *x.color());
        if color.is_none() {
            return;
        }

        let color = color.unwrap();

        let mut poses = vec![];

        // Now, iterate over vehicles for this team that are alive and spawn fireworks.
        for (entity, unit) in world.component_iter::<components::unit::Unit>() {
            if let Some(unit_team) = world.component::<components::team_member::TeamMember>(entity)
            {
                if unit_team.team() == winner_team_id {
                    if let Some(health) = world.component::<components::health::Health>(entity) {
                        if health.is_destroyed() {
                            continue;
                        }
                        // now, we need to retrieve the real pose...
                        let position_entity = match unit.unit_type() {
                            UnitType::Artillery => {
                                world
                                    .component::<units::artillery::UnitArtillery>(entity)
                                    .expect("unit should exist")
                                    .turret_entity
                            }
                            UnitType::Tank => {
                                world
                                    .component::<units::tank::UnitTank>(entity)
                                    .expect("unit should exist")
                                    .turret_entity
                            }
                            _ => {
                                continue;
                            }
                        };
                        let pose = components::pose::world_pose(world, position_entity);
                        poses.push(cgmath::vec3(pose.x(), pose.y(), pose.z()));
                    }
                }
            }
        }

        for (i, v) in poses.iter().enumerate() {
            crate::display::fireworks::create_firework(*v, color, world, i);
        }
    }
}
