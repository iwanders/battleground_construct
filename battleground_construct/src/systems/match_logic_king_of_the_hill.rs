use crate::components;
use components::capturable::Capturable;
use components::capture_point::CapturePoint;
use components::clock::Clock;
use components::match_king_of_the_hill::MatchKingOfTheHill;
use components::team::TeamId;

use engine::prelude::*;

pub struct MatchLogicKingOfTheHill {}
impl System for MatchLogicKingOfTheHill {
    fn update(&mut self, world: &mut World) {
        let dt = world
            .component_iter_mut::<Clock>()
            .next()
            .expect("Should have one clock")
            .1
            .step_as_f32();

        let mut owners: std::collections::HashMap<TeamId, f32> = Default::default();

        for (e, capturable) in world.component_iter::<Capturable>() {
            if let Some(_v) = world.component::<CapturePoint>(e) {
                // Only if it is a capture point, add dt for this control point.
                // this makes it a trivial extension to make different control points accrue
                // differently.
                if let Some(team) = capturable.owner() {
                    *owners.entry(team).or_insert(0.0) += dt;
                }
            }
        }

        if let Some((_e, mut koth)) = world.component_iter_mut::<MatchKingOfTheHill>().next() {
            let update_pairs = owners
                .iter()
                .map(|(t, v)| (*t, *v))
                .collect::<Vec<(TeamId, f32)>>();
            koth.add_points(&update_pairs);
        }
    }
}
