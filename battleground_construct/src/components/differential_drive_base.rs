use engine::prelude::*;

#[derive(Copy, Debug, Clone)]
pub struct DifferentialDriveBase {
    pub track_width: f32,
    pub wheel_velocity_bounds: (f32, f32),
    pub wheel_velocity: (f32, f32),
}
impl Default for DifferentialDriveBase {
    fn default() -> Self {
        DifferentialDriveBase::new()
    }
}

impl DifferentialDriveBase {
    pub fn new() -> Self {
        DifferentialDriveBase {
            track_width: 1.0,
            wheel_velocity_bounds: (-1.0, 1.0),
            wheel_velocity: (0.0, 0.0),
        }
    }

    pub fn set_velocities(&mut self, left: f32, right: f32) {
        self.wheel_velocity = (
            left.clamp(self.wheel_velocity_bounds.0, self.wheel_velocity_bounds.1),
            right.clamp(self.wheel_velocity_bounds.0, self.wheel_velocity_bounds.1),
        );
    }

    pub fn track_width(&self) -> f32 {
        self.track_width
    }

    pub fn wheel_velocities(&self) -> (f32, f32) {
        self.wheel_velocity
    }

    pub fn wheel_velocity_bounds(&mut self) -> (f32, f32) {
        self.wheel_velocity_bounds
    }
}
impl Component for DifferentialDriveBase {}

use crate::components::unit_interface::{Register, RegisterMap, UnitModule};
use battleground_unit_control::modules::differential_drive::*;
pub struct DifferentialDriveBaseModule {
    entity: EntityId,
}

impl DifferentialDriveBaseModule {
    pub fn new(entity: EntityId) -> Self {
        DifferentialDriveBaseModule { entity }
    }
}

impl UnitModule for DifferentialDriveBaseModule {
    fn get_registers(&self, world: &World, registers: &mut RegisterMap) {
        registers.clear();
        if let Some(base) = world.component::<DifferentialDriveBase>(self.entity) {
            let vels = base.wheel_velocities();
            registers.insert(
                REG_DIFF_DRIVE_LEFT_VEL,
                Register::new_f32("left_wheel_vel", vels.0),
            );
            registers.insert(
                REG_DIFF_DRIVE_RIGHT_VEL,
                Register::new_f32("right_wheel_vel", vels.1),
            );

            // commanded is the same as reported atm.
            registers.insert(
                REG_DIFF_DRIVE_LEFT_CMD,
                Register::new_f32("left_wheel_cmd", vels.0),
            );
            registers.insert(
                REG_DIFF_DRIVE_RIGHT_CMD,
                Register::new_f32("right_wheel_cmd", vels.1),
            );

            registers.insert(
                REG_DIFF_DRIVE_TRACK_WIDTH,
                Register::new_f32("track_width", base.track_width()),
            );
        }
    }

    fn set_component(&self, world: &mut World, registers: &RegisterMap) {
        if let Some(mut base) = world.component_mut::<DifferentialDriveBase>(self.entity) {
            let left_cmd = registers
                .get(&REG_DIFF_DRIVE_LEFT_CMD)
                .expect("register doesnt exist")
                .value_f32()
                .expect("wrong value type"); // denotes mismatch between get_registers and set_component.
            let right_cmd = registers
                .get(&REG_DIFF_DRIVE_RIGHT_CMD)
                .expect("register doesnt exist")
                .value_f32()
                .expect("wrong value type"); // denotes mismatch between get_registers and set_component.
            base.set_velocities(left_cmd, right_cmd);
        }
    }
}
