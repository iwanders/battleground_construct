use engine::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Copy, Debug, Clone)]
pub struct DifferentialDriveConfig {
    pub track_width: f32,
    #[serde(skip)]
    pub wheel_velocity_bounds: (f32, f32),
    #[serde(skip)]
    pub wheel_acceleration_bounds: Option<(f32, f32)>,
}

impl Default for DifferentialDriveConfig {
    fn default() -> Self {
        DifferentialDriveConfig {
            track_width: 1.0,
            wheel_velocity_bounds: (-1.0, 1.0),
            wheel_acceleration_bounds: Some((-0.5, 0.5)),
        }
    }
}

#[derive(Deserialize, Serialize, Copy, Debug, Clone)]
pub struct DifferentialDriveBase {
    config: DifferentialDriveConfig,
    #[serde(skip)]
    wheel_velocity_cmd: (f32, f32),
    wheel_velocity_vel: (f32, f32),
}
impl Default for DifferentialDriveBase {
    fn default() -> Self {
        DifferentialDriveBase::new()
    }
}

impl DifferentialDriveBase {
    pub fn new() -> Self {
        Self::from_config(Default::default())
    }

    pub fn from_config(config: DifferentialDriveConfig) -> Self {
        DifferentialDriveBase {
            config,
            wheel_velocity_cmd: (0.0, 0.0),
            wheel_velocity_vel: (0.0, 0.0),
        }
    }

    pub fn set_velocities(&mut self, left: f32, right: f32) {
        self.wheel_velocity_cmd = (
            left.clamp(
                self.config.wheel_velocity_bounds.0,
                self.config.wheel_velocity_bounds.1,
            ),
            right.clamp(
                self.config.wheel_velocity_bounds.0,
                self.config.wheel_velocity_bounds.1,
            ),
        );
    }

    pub fn track_width(&self) -> f32 {
        self.config.track_width
    }

    pub fn wheel_velocities(&self) -> (f32, f32) {
        self.wheel_velocity_vel
    }
    pub fn wheel_velocities_cmd(&self) -> (f32, f32) {
        self.wheel_velocity_cmd
    }

    pub fn wheel_velocities_mut(&mut self) -> &mut (f32, f32) {
        &mut self.wheel_velocity_vel
    }

    pub fn wheel_velocity_bounds(self) -> (f32, f32) {
        self.config.wheel_velocity_bounds
    }

    pub fn wheel_acceleration_bounds(&self) -> Option<(f32, f32)> {
        self.config.wheel_acceleration_bounds
    }

    /// Apply the acceleration limits.
    pub fn update(&mut self, dt: f32) {
        if let Some(ref bounds) = self.config.wheel_acceleration_bounds {
            // Calculate the desired acceleration.
            let left_desired_accel = (self.wheel_velocity_cmd.0 - self.wheel_velocity_vel.0) / dt;
            let right_desired_accel = (self.wheel_velocity_cmd.1 - self.wheel_velocity_vel.1) / dt;
            // clamp the acceleration based on the limits, and integrate with time to get the actual
            // velocity change.
            self.wheel_velocity_vel.0 += dt * left_desired_accel.clamp(bounds.0, bounds.1);
            self.wheel_velocity_vel.1 += dt * right_desired_accel.clamp(bounds.0, bounds.1);
        } else {
            // no acceleration limits.
            self.wheel_velocity_vel = self.wheel_velocity_cmd;
        }
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

            let cmd = base.wheel_velocities_cmd();

            registers.insert(
                REG_DIFF_DRIVE_LEFT_CMD,
                Register::new_f32("left_wheel_cmd", cmd.0),
            );
            registers.insert(
                REG_DIFF_DRIVE_RIGHT_CMD,
                Register::new_f32("right_wheel_cmd", cmd.1),
            );

            let accel_bounds = base.wheel_acceleration_bounds().unwrap_or((0.0, 0.0));
            registers.insert(
                REG_DIFF_DRIVE_ACCELERATION_LOWER,
                Register::new_f32("acceleration_lower", accel_bounds.0),
            );
            registers.insert(
                REG_DIFF_DRIVE_ACCELERATION_UPPER,
                Register::new_f32("acceleration_upper", accel_bounds.1),
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
