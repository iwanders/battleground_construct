use engine::prelude::*;

#[derive(Copy, Debug, Clone)]
pub struct DifferentialDriveBase {
    pub track_width: f32,
    pub wheel_velocity_bounds: (f32, f32),
    pub wheel_velocity: (f32, f32),
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

use crate::components::vehicle_interface::{Register, RegisterMap, VehicleModule};
pub struct DifferentialDriveBaseControl {
    entity: EntityId,
}

impl DifferentialDriveBaseControl {
    pub fn new(entity: EntityId) -> Self {
        DifferentialDriveBaseControl { entity }
    }
}

impl VehicleModule for DifferentialDriveBaseControl {
    fn get_registers(&self, world: &World, registers: &mut RegisterMap) {
        registers.clear();
        if let Some(base) = world.component::<DifferentialDriveBase>(self.entity) {
            let vels = base.wheel_velocities();
            registers.insert(0, Register::new_f32("left_wheel_vel", vels.0));
            registers.insert(1, Register::new_f32("right_wheel_vel", vels.1));

            // commanded is the same as reported atm.
            registers.insert(2, Register::new_f32("left_wheel_cmd", vels.0));
            registers.insert(3, Register::new_f32("right_wheel_cmd", vels.1));
        }
    }

    fn set_component(&self, world: &mut World, registers: &RegisterMap) {
        if let Some(mut base) = world.component_mut::<DifferentialDriveBase>(self.entity) {
            let left_cmd = registers
                .get(&2)
                .expect("register doesnt exist")
                .value_f32()
                .expect("wrong value type"); // denotes mismatch between get_registers and set_component.
            let right_cmd = registers
                .get(&3)
                .expect("register doesnt exist")
                .value_f32()
                .expect("wrong value type"); // denotes mismatch between get_registers and set_component.
            println!("Setting {left_cmd} and {right_cmd}");
            base.set_velocities(left_cmd, right_cmd);
        }
    }
}
