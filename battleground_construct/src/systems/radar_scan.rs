use super::components::pose::Pose;
use super::components::radar::Radar;
use super::components::radar_reflector::RadarReflector;
use crate::display::primitives::Mat4;
use engine::prelude::*;

pub struct RadarScan {}
impl System for RadarScan {
    fn update(&mut self, world: &mut World) {
        let mut reflectors: Vec<(Mat4, f32)> = vec![];
        for (entity, reflector) in world.component_iter::<RadarReflector>() {
            if let Some(pose) = world.component::<Pose>(entity) {
                reflectors.push((*pose.transform(), reflector.reflectivity()));
            }
        }

        for (entity, mut radar) in world.component_iter_mut::<Radar>() {
            if let Some(pose) = world.component::<Pose>(entity) {
                let radar_pose = pose.transform();
                radar.update_reflections(radar_pose, &reflectors);
            }
        }
    }
}
