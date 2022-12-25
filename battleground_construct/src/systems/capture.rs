use super::components;
use crate::components::team::TeamId;
use components::capturable::Capturable;
use components::capture_marker::CaptureMarker;
use components::capture_point::CapturePoint;
use components::team_member::TeamMember;

use crate::util::cgmath::prelude::*;

use super::Clock;
use engine::prelude::*;

pub struct Capture {}
impl System for Capture {
    fn update(&mut self, world: &mut World) {
        let dt = {
            let (_entity, clock) = world
                .component_iter_mut::<Clock>()
                .next()
                .expect("Should have one clock");
            clock.step_as_f32()
        };

        for (capturable_entity, mut capturable) in world.component_iter_mut::<Capturable>() {
            let mut influence = std::collections::HashMap::new();
            if let Some(capture_point) = world.component::<CapturePoint>(capturable_entity) {
                let point_pose = components::pose::world_pose(world, capturable_entity);
                for (marker_entity, _marker) in world.component_iter::<CaptureMarker>() {
                    if let Some(team_membership) = world.component::<TeamMember>(marker_entity) {
                        let marker_pose = components::pose::world_pose(world, marker_entity);
                        if (point_pose.to_translation() - marker_pose.to_translation())
                            .euclid_norm()
                            < capture_point.radius()
                        {
                            *influence.entry(team_membership.team()).or_insert(0.0) +=
                                1.0 * dt * capture_point.speed();
                        }
                    }
                }
            }
            let influence_vec: Vec<(TeamId, f32)> =
                influence.iter().map(|(a, b)| (*a, *b)).collect::<_>();
            capturable.update(&influence_vec[..]);
            // println!("capturable: {capturable:?}");
        }
    }
}
