use super::components::group::Group;
use super::components::pose::world_pose;
use super::components::radar::Radar;
use super::components::radar_reflector::RadarReflector;
use crate::display::primitives::Mat4;
use engine::prelude::*;

pub struct RadarScan {}
impl System for RadarScan {
    fn update(&mut self, world: &mut World) {
        let mut reflectors: Vec<(Mat4, f32, Group)> = vec![];
        for (entity, reflector) in world.component_iter::<RadarReflector>() {
            let pose = world_pose(world, entity);
            reflectors.push((
                *pose.transform(),
                reflector.reflectivity(),
                world.component::<Group>(entity).unwrap().clone(),
            ));
        }

        for (entity, mut radar) in world.component_iter_mut::<Radar>() {
            let radar_pose = world_pose(world, entity);
            let reflectors = reflectors
                .iter()
                .filter(|v| !v.2.entities().contains(&entity))
                .map(|v| (v.0, v.1))
                .collect::<Vec<_>>();
            radar.update_reflections(&radar_pose, &reflectors);
        }
    }
}
