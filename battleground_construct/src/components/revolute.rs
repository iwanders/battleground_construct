use crate::components::pose::Pose;
use engine::prelude::*;

#[derive(Copy, Debug, Clone)]
pub struct Revolute {
    pub velocity_bounds: (f32, f32),
    pub velocity: f32,
    pub position: f32,
    pub axis: cgmath::Vector3<f32>,
}
impl Default for Revolute {
    fn default() -> Self {
        Revolute::new()
    }
}

impl Revolute {
    pub fn new() -> Self {
        Self::new_with_axis(cgmath::Vector3::<f32>::new(1.0, 0.0, 0.0))
    }
    pub fn new_with_axis(axis: cgmath::Vector3<f32>) -> Self {
        Revolute {
            velocity: 0.0,
            position: 0.0,
            axis,
            velocity_bounds: (-1.0, 1.0),
        }
    }

    pub fn set_axis(&mut self, axis: cgmath::Vector3<f32>) {
        self.axis = axis;
    }

    pub fn set_velocity(&mut self, velocity: f32) {
        self.velocity = velocity.clamp(self.velocity_bounds.0, self.velocity_bounds.1);
    }

    pub fn velocity(&self) -> f32 {
        self.velocity
    }

    pub fn position(&self) -> f32 {
        self.position
    }

    pub fn set_position(&mut self, value: f32) {
        self.position = value
    }

    pub fn velocity_bounds(&self) -> (f32, f32) {
        self.velocity_bounds
    }

    pub fn set_velocity_bounds(&mut self, min: f32, max: f32) {
        self.velocity_bounds = (min, max)
    }

    pub fn integrate(&mut self, dt: f32) {
        self.position = (self.position + dt * self.velocity).rem_euclid(std::f32::consts::PI * 2.0);
    }

    pub fn to_pose(&self) -> Pose {
        cgmath::Matrix4::<f32>::from_axis_angle(self.axis, cgmath::Rad(self.position)).into()
    }

    pub fn to_twist(&self) -> crate::util::cgmath::prelude::Twist<f32> {
        crate::util::cgmath::prelude::Twist::<f32>::new(
            cgmath::Vector3::<f32>::new(0.0, 0.0, 0.0),
            self.axis * self.velocity,
        )
    }
}
impl Component for Revolute {}

use crate::components::vehicle_interface::{Register, RegisterMap, VehicleModule};
pub struct RevoluteModule {
    entity: EntityId,
}

impl RevoluteModule {
    pub fn new(entity: EntityId) -> Self {
        RevoluteModule { entity }
    }
}

impl VehicleModule for RevoluteModule {
    fn get_registers(&self, world: &World, registers: &mut RegisterMap) {
        registers.clear();
        if let Some(revolute) = world.component::<Revolute>(self.entity) {
            registers.insert(0, Register::new_f32("position", revolute.position()));
            registers.insert(1, Register::new_f32("velocity", revolute.velocity()));

            let (vel_min, vel_max) = revolute.velocity_bounds();
            registers.insert(2, Register::new_f32("velocity_min", vel_min));
            registers.insert(3, Register::new_f32("velocity_max", vel_max));

            registers.insert(4, Register::new_f32("velocity_cmd", revolute.velocity()));
        }
    }

    fn set_component(&self, world: &mut World, registers: &RegisterMap) {
        if let Some(mut revolute) = world.component_mut::<Revolute>(self.entity) {
            let vel_cmd = registers
                .get(&4)
                .expect("register doesnt exist")
                .value_f32()
                .expect("wrong value type");
            revolute.set_velocity(vel_cmd);
        }
    }
}
