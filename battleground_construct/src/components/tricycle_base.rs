use engine::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Copy, Debug, Clone)]
pub struct TricycleConfig {
    pub wheel_base: f32,
    #[serde(skip)]
    pub wheel_velocity_bounds: (f32, f32),
    #[serde(skip)]
    pub wheel_acceleration_bounds: Option<(f32, f32)>,
}

impl Default for TricycleConfig {
    fn default() -> Self {
        TricycleConfig {
            wheel_base: 1.0,
            wheel_velocity_bounds: (-1.0, 1.0),
            wheel_acceleration_bounds: Some((-0.5, 0.5)),
        }
    }
}

#[derive(Deserialize, Serialize, Copy, Debug, Clone)]
pub struct TricycleBase {
    config: TricycleConfig,
    #[serde(skip)]
    wheel_velocity_cmd: f32,
    wheel_velocity_vel: f32,

    // #[serde(skip)]
    steering_joint: EntityId,
}

impl TricycleBase {
    pub fn new(config: TricycleConfig, steering_joint: EntityId) -> Self {
        TricycleBase {
            steering_joint,
            config,
            wheel_velocity_cmd: 0.0,
            wheel_velocity_vel: 0.0,
        }
    }

    pub fn set_velocity(&mut self, desired: f32) {
        self.wheel_velocity_cmd = desired.clamp(
            self.config.wheel_velocity_bounds.0,
            self.config.wheel_velocity_bounds.1,
        );
    }

    pub fn wheel_base(&self) -> f32 {
        self.config.wheel_base
    }

    pub fn wheel_velocity_cmd(&self) -> f32 {
        self.wheel_velocity_cmd
    }

    pub fn wheel_velocity(&self) -> f32 {
        self.wheel_velocity_vel
    }
    pub fn steering_joint(&self) -> EntityId {
        self.steering_joint
    }

    pub fn wheel_velocities_mut(&mut self) -> &mut f32 {
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
            let desired_accel = (self.wheel_velocity_cmd - self.wheel_velocity_vel) / dt;
            // clamp the acceleration based on the limits, and integrate with time to get the actual
            // velocity change.
            self.wheel_velocity_vel += dt * desired_accel.clamp(bounds.0, bounds.1);
        } else {
            // no acceleration limits.
            self.wheel_velocity_vel = self.wheel_velocity_cmd;
        }
    }
}
impl Component for TricycleBase {}

use crate::components::unit_interface::{Register, RegisterMap, UnitModule};
use battleground_unit_control::modules::differential_drive::*;
pub struct TricycleBaseModule {
    entity: EntityId,
}

impl TricycleBaseModule {
    pub fn new(entity: EntityId) -> Self {
        TricycleBaseModule { entity }
    }
}

impl UnitModule for TricycleBaseModule {
    fn get_registers(&self, world: &World, registers: &mut RegisterMap) {
        registers.clear();
        if let Some(base) = world.component::<TricycleBase>(self.entity) {
            let vels = base.wheel_velocity();

            registers.insert(
                REG_DIFF_DRIVE_LEFT_VEL,
                Register::new_f32("wheel_vel", vels),
            );

            let cmd = base.wheel_velocity_cmd();
            registers.insert(REG_DIFF_DRIVE_LEFT_CMD, Register::new_f32("wheel_cmd", cmd));

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
                Register::new_f32("wheel_base", base.wheel_base()),
            );
        }
    }

    fn set_component(&self, world: &mut World, registers: &RegisterMap) {
        if let Some(mut base) = world.component_mut::<TricycleBase>(self.entity) {
            let cmd = registers
                .get(&REG_DIFF_DRIVE_LEFT_CMD)
                .expect("register doesnt exist")
                .value_f32()
                .expect("wrong value type"); // denotes mismatch between get_registers and set_component.
            base.set_velocity(cmd);
        }
    }
}
