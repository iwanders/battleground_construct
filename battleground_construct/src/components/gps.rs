use super::pose::world_pose;
use engine::prelude::*;

use battleground_unit_control::modules::gps::registers as gps_registers;

use crate::components::unit_interface::{Register, RegisterMap, UnitModule};
pub struct GpsModule {
    entity: EntityId,
}

impl GpsModule {
    pub fn new(entity: EntityId) -> Self {
        GpsModule { entity }
    }
}

impl UnitModule for GpsModule {
    fn get_registers(&self, world: &World, registers: &mut RegisterMap) {
        use crate::util::cgmath::ToRollPitchYaw;
        use crate::util::cgmath::ToTranslation;
        let pose = world_pose(world, self.entity);
        let translation = pose.to_translation();
        registers.insert(gps_registers::X, Register::new_f32("x", translation.x));
        registers.insert(gps_registers::Y, Register::new_f32("y", translation.y));
        registers.insert(gps_registers::Z, Register::new_f32("z", translation.z));

        let rpy = pose.to_rpy();
        registers.insert(gps_registers::ROLL, Register::new_f32("r", rpy.x));
        registers.insert(gps_registers::PITCH, Register::new_f32("p", rpy.y));
        registers.insert(gps_registers::YAW, Register::new_f32("y", rpy.z));
    }
}
