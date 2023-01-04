use crate::components::pose::Pose;
use engine::prelude::*;

#[derive(Copy, Debug, Clone)]
pub struct RevoluteConfig {
    pub velocity_bounds: (f32, f32),
    pub acceleration_bounds: Option<(f32, f32)>,
    pub velocity: f32,
    pub velocity_cmd: f32,
    pub position: f32,
    pub axis: cgmath::Vector3<f32>,
}
impl Default for RevoluteConfig {
    fn default() -> Self {
        Self {
            velocity: 0.0,
            velocity_cmd: 0.0,
            position: 0.0,
            velocity_bounds: (-1.0, 1.0),
            acceleration_bounds: None,
            axis: cgmath::Vector3::<f32>::new(1.0, 0.0, 0.0),
        }
    }
}

#[derive(Copy, Debug, Clone)]
pub struct Revolute {
    config: RevoluteConfig,
    velocity_cmd: f32,
    velocity: f32,
    position: f32,
}
impl Default for Revolute {
    fn default() -> Self {
        Revolute::new()
    }
}

impl Revolute {
    pub fn new() -> Self {
        Self::from_config(Default::default())
    }
    pub fn new_with_axis(axis: cgmath::Vector3<f32>) -> Self {
        Self::from_config(RevoluteConfig {
            axis,
            ..Default::default()
        })
    }
    pub fn from_config(config: RevoluteConfig) -> Self {
        Revolute {
            velocity: config
                .velocity
                .clamp(config.velocity_bounds.0, config.velocity_bounds.1),
            position: config.position,
            velocity_cmd: config.velocity_cmd,
            config,
        }
    }

    pub fn set_axis(&mut self, axis: cgmath::Vector3<f32>) {
        self.config.axis = axis;
    }

    pub fn set_velocity_cmd(&mut self, velocity: f32) {
        self.velocity_cmd =
            velocity.clamp(self.config.velocity_bounds.0, self.config.velocity_bounds.1);
    }

    pub fn velocity(&self) -> f32 {
        self.velocity
    }

    pub fn velocity_cmd(&self) -> f32 {
        self.velocity_cmd
    }

    pub fn position(&self) -> f32 {
        self.position
    }

    pub fn set_position(&mut self, value: f32) {
        self.position = value
    }

    pub fn velocity_bounds(&self) -> (f32, f32) {
        self.config.velocity_bounds
    }

    pub fn acceleration_bounds(&self) -> Option<(f32, f32)> {
        self.config.acceleration_bounds
    }

    pub fn set_velocity_bounds(&mut self, min: f32, max: f32) {
        self.config.velocity_bounds = (min, max)
    }

    pub fn update(&mut self, dt: f32) {
        if let Some(ref bound) = self.config.acceleration_bounds {
            // Calculate the desired acceleration.
            let desired_accel = (self.velocity_cmd - self.velocity) / dt;
            // clamp the acceleration based on the limits, and integrate with time to get the actual
            // velocity change.
            self.velocity += dt * desired_accel.clamp(bound.0, bound.1);
        } else {
            self.velocity = self.velocity_cmd;
        }
        self.position = (self.position + dt * self.velocity).rem_euclid(std::f32::consts::PI * 2.0);
    }

    /// Copy the velocity cmd to the velocity, bypassing any acceleration bounds.
    pub fn set_velocity_to_cmd(&mut self) {
        self.velocity = self.velocity_cmd;
    }

    pub fn to_pose(&self) -> Pose {
        cgmath::Matrix4::<f32>::from_axis_angle(self.config.axis, cgmath::Rad(self.position)).into()
    }

    pub fn to_twist(&self) -> crate::util::cgmath::prelude::Twist<f32> {
        crate::util::cgmath::prelude::Twist::<f32>::new(
            cgmath::Vector3::<f32>::new(0.0, 0.0, 0.0),
            self.config.axis * self.velocity,
        )
    }
}
impl Component for Revolute {}

use crate::components::unit_interface::{Register, RegisterMap, UnitModule};
use battleground_unit_control::modules::revolute::*;
pub struct RevoluteModule {
    entity: EntityId,
}

impl RevoluteModule {
    pub fn new(entity: EntityId) -> Self {
        RevoluteModule { entity }
    }
}

impl UnitModule for RevoluteModule {
    fn get_registers(&self, world: &World, registers: &mut RegisterMap) {
        registers.clear();
        if let Some(revolute) = world.component::<Revolute>(self.entity) {
            registers.insert(
                REG_REVOLUTE_POSITION,
                Register::new_f32("position", revolute.position()),
            );
            registers.insert(
                REG_REVOLUTE_VELOCITY,
                Register::new_f32("velocity", revolute.velocity()),
            );

            let (vel_min, vel_max) = revolute.velocity_bounds();
            registers.insert(
                REG_REVOLUTE_VELOCITY_MIN,
                Register::new_f32("velocity_min", vel_min),
            );
            registers.insert(
                REG_REVOLUTE_VELOCITY_MAX,
                Register::new_f32("velocity_max", vel_max),
            );

            registers.insert(
                REG_REVOLUTE_VELOCITY_CMD,
                Register::new_f32("velocity_cmd", revolute.velocity_cmd()),
            );

            let accel_bounds = revolute.acceleration_bounds().unwrap_or((0.0, 0.0));
            registers.insert(
                REG_REVOLUTE_ACCELERATION_LOWER,
                Register::new_f32("acceleration_lower", accel_bounds.0),
            );
            registers.insert(
                REG_REVOLUTE_ACCELERATION_UPPER,
                Register::new_f32("acceleration_upper", accel_bounds.1),
            );
        }
    }

    fn set_component(&self, world: &mut World, registers: &RegisterMap) {
        if let Some(mut revolute) = world.component_mut::<Revolute>(self.entity) {
            let vel_cmd = registers
                .get(&REG_REVOLUTE_VELOCITY_CMD)
                .expect("register doesnt exist")
                .value_f32()
                .expect("wrong value type");
            revolute.set_velocity_cmd(vel_cmd);
        }
    }
}
