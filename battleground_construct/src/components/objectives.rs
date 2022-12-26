use super::pose::world_pose;
use engine::prelude::*;

use crate::components;
use components::capturable::Capturable;
use components::capture_point::CapturePoint;
// use components::match_king_of_the_hill::MatchKingOfTheHill;

use crate::components::unit_interface::{Register, RegisterMap, UnitModule};

pub struct ObjectivesModule {}
impl ObjectivesModule {
    pub fn new() -> Self {
        ObjectivesModule {}
    }
}

use battleground_unit_control::modules::objectives::register;

impl UnitModule for ObjectivesModule {
    fn get_registers(&self, world: &World, registers: &mut RegisterMap) {
        registers.clear();

        // unit control actually doesn't need to know about the game type? Maybe they do... shrug.
        // if let Some((_e, mut koth)) = world.component_iter_mut::<MatchKingOfTheHill>().next() {
        // }

        // Collect the capture points into a vector holding their relevant information.
        let mut capture_points = vec![];
        for (e, capturable) in world.component_iter::<Capturable>() {
            if let Some(_v) = world.component::<CapturePoint>(e) {
                use crate::util::cgmath::ToTranslation;
                let pose = world_pose(world, e).to_translation();
                capture_points.push((pose.x, pose.y, capturable.owner()));
            }
        }

        registers.insert(
            register::CAPTURE_POINT_COUNT,
            Register::new_i32("capture_point_count", capture_points.len() as i32),
        );
        let record_per_point = 3;
        for (i, (x, y, owner)) in capture_points.iter().enumerate() {
            registers.insert(
                register::CAPTURE_POINT_COUNT + (i * record_per_point) as u32,
                Register::new_f32("x", *x),
            );
            registers.insert(
                register::CAPTURE_POINT_COUNT + (i * record_per_point) as u32 + 1,
                Register::new_f32("y", *y),
            );
            let owner_value = owner.map(|v| v.as_u64() as i32).unwrap_or(-1);
            registers.insert(
                register::CAPTURE_POINT_COUNT + (i * record_per_point) as u32 + 2,
                Register::new_i32("owner", owner_value),
            );
        }
    }
}
