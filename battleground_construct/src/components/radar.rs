use crate::display::primitives::Mat4;
use engine::prelude::*;

#[derive(Copy, Debug, Clone)]
pub struct Reflection {
    pub yaw: f32,
    pub pitch: f32,
    pub strength: f32,
}

#[derive(Debug, Clone)]
pub struct Radar {
    pub range_max: f32,
    pub signal_strength: f32,
    pub cone_angle: f32,
    pub reflections: Vec<Reflection>,
}

impl Radar {
    pub fn new() -> Self {
        Self {
            range_max: 30.0,
            cone_angle: 1.0f32.to_radians(),
            signal_strength: 1.0,
            reflections: vec![],
        }
    }

    pub fn reflections(&self) -> Vec<Reflection> {
        self.reflections.clone()
    }

    pub fn update_reflections(&mut self, radar_pose: &Mat4, reflectors: &[(Mat4, f32)]) {}
}
impl Component for Radar {}

use crate::components::vehicle_interface::{Register, RegisterMap, VehicleModule};
pub struct RadarControl {
    entity: EntityId,
}

impl RadarControl {
    pub fn new(entity: EntityId) -> Self {
        RadarControl { entity }
    }
}

impl VehicleModule for RadarControl {
    fn get_registers(&self, world: &World, registers: &mut RegisterMap) {
        registers.clear();
        if let Some(radar) = world.component::<Radar>(self.entity) {
            let reflections = radar.reflections();
            registers.insert(
                0,
                Register::new_i32("reflections", reflections.len() as i32),
            );
        }
    }
}
