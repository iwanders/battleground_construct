use crate::display::primitives::Mat4;
use engine::prelude::*;

#[derive(Copy, Debug, Clone)]
pub struct Reflection {
    pub yaw: f32,
    pub pitch: f32,
    pub strength: f32,
    pub distance: f32,
}

#[derive(Debug, Clone)]
pub struct Radar {
    range_max: f32,
    signal_strength: f32,
    detection_angle_yaw: f32,
    detection_angle_pitch: f32,

    reflections: Vec<Reflection>,
}

#[derive(Copy, Debug, Clone)]
pub struct RadarConfig {
    /// Maximum range of this radar, distances beyond this can never be seen.
    pub range_max: f32,
    /// Maximum detection angle in yaw. (Symmetric around local x, everywhere is pi/2)
    pub detection_angle_yaw: f32,
    /// Maximum detection angle in pitch. (Symmetric around local x, everywhere is pi/2)
    pub detection_angle_pitch: f32,
    /// Signal strength that's emitted by the radar (follows inverse square law, and respects reflectivity.
    pub signal_strength: f32,
}
impl Default for RadarConfig {
    fn default() -> Self {
        Self {
            range_max: 30.0,
            detection_angle_yaw: 1.0f32.to_radians(),
            detection_angle_pitch: 180f32.to_radians(),
            signal_strength: 1.0,
        }
    }
}

impl Radar {
    pub fn new_with_config(config: RadarConfig) -> Self {
        Self {
            reflections: vec![],
            range_max: config.range_max,
            detection_angle_yaw: config.detection_angle_yaw,
            detection_angle_pitch: config.detection_angle_pitch,
            signal_strength: config.signal_strength,
        }
    }

    pub fn reflections(&self) -> Vec<Reflection> {
        self.reflections.clone()
    }

    pub fn update_reflections(&mut self, radar_pose: &Mat4, reflectors: &[(Mat4, f32)]) {
        use crate::util::cgmath::prelude::*;
        self.reflections.clear();
        for (pos, reflectivity) in reflectors.iter() {
            let pos_v = pos.to_translation();
            let radar_v = radar_pose.to_translation();
            let distance = radar_v.distance2(pos_v).sqrt();
            if distance >= self.range_max {
                continue; // so far away, it's out of range, easy optimisation.
            }

            // It could be in range... now we need to do math.
            // Express the reflector pose in the radar's frame.
            // radar pose is world -> radar
            // reflector is world -> reflector
            // we want radar -> reflector
            let reflector_local = radar_pose.to_inv_h() * pos;

            // Now the reflector is in local radar frame.
            // Calculating yaw and pitch is now easy.
            let local = reflector_local.to_translation();
            let distance = local.euclid_norm();

            let yaw = local.y.atan2(local.x); // atan2 returns in -pi/2, pi/2
            let pitch = (local.z / distance).asin();

            let inside_yaw = yaw.abs() <= self.detection_angle_yaw;
            let inside_pitch = pitch.abs() <= self.detection_angle_pitch;

            if inside_yaw && inside_pitch {
                // Calculate reflectivity,
                let ratio_towards = 1.0 / distance.powi(2);
                let reflected = reflectivity * ratio_towards;
                let ratio_back = 1.0 / distance.powi(2);
                let strength = ratio_back * reflected * self.signal_strength;
                self.reflections.push(Reflection {
                    yaw,
                    pitch,
                    strength,
                    distance,
                });
            }
        }
    }
}
impl Component for Radar {}

use crate::components::vehicle_interface::{Register, RegisterMap, VehicleModule};
pub struct RadarModule {
    entity: EntityId,
}

impl RadarModule {
    pub fn new(entity: EntityId) -> Self {
        RadarModule { entity }
    }
}

impl VehicleModule for RadarModule {
    fn get_registers(&self, world: &World, registers: &mut RegisterMap) {
        registers.clear();
        if let Some(radar) = world.component::<Radar>(self.entity) {
            let reflections = radar.reflections();
            let mut index = 0;
            registers.insert(index, Register::new_f32("range_max", radar.range_max));
            index += 1;
            registers.insert(
                index,
                Register::new_f32("detection_angle_yaw", radar.detection_angle_yaw),
            );
            index += 1;
            registers.insert(
                index,
                Register::new_f32("detection_angle_pitch", radar.detection_angle_pitch),
            );
            index += 1;
            registers.insert(
                index,
                Register::new_i32("reflections", reflections.len() as i32),
            );
            index += 1;
            for (i, reflection) in reflections.iter().enumerate() {
                let offset = i as u32 * 4 + index;
                registers.insert(offset, Register::new_f32("yaw", reflection.yaw));
                registers.insert(offset + 1, Register::new_f32("pitch", reflection.pitch));
                registers.insert(
                    offset + 2,
                    Register::new_f32("distance", reflection.distance),
                );
                registers.insert(
                    offset + 3,
                    Register::new_f32("strength", reflection.strength),
                );
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::util::test_util::approx_equal;
    use cgmath::vec3;
    #[test]
    fn test_radar_normal_update() {
        let mut radar = Radar::new_with_config(RadarConfig {
            range_max: 30.0,
            detection_angle_yaw: 1.0f32.to_radians(),
            detection_angle_pitch: 180f32.to_radians(),
            signal_strength: 1.0,
        });
        let reflections = vec![
            (Mat4::from_translation(vec3(5.0f32, 3.0, 0.0)), 1.0), // seen! 10m
            (Mat4::from_translation(vec3(50.0f32, 3.0, 0.0)), 1.0), // not seen, too far.
            (Mat4::from_translation(vec3(20.0f32, 3.01, 0.0)), 1.0), // Seen ~25m
            (Mat4::from_translation(vec3(-4.99f32, 3.00, 25.0)), 1.0), // Seen ~25m, high pitch.
            (Mat4::from_translation(vec3(-4.99f32, 3.00, -25.0)), 1.0), // Seen ~25m, low pitch.
        ];
        let radar_pose = Mat4::from_translation(vec3(-5.0f32, 3.0, 0.0));
        radar.update_reflections(&radar_pose, &reflections);

        let expected = vec![
            (0.0f32, 0.0f32, 10.0f32),
            (0.0, 0.0, 25.0),
            (0.0, 1.5707964, 25.0),
            (0.0, -1.5707964, 25.0),
        ];
        let obtained = radar.reflections();
        println!("Obtained: {obtained:?}");
        assert_eq!(obtained.len(), expected.len());
        for (obtain, expect) in obtained.iter().zip(expected.iter()) {
            approx_equal!(obtain.yaw, expect.0, 0.001);
            approx_equal!(obtain.pitch, expect.1, 0.001);
            approx_equal!(obtain.distance, expect.2, 0.001);
        }
    }
    #[test]
    fn test_radar_spherical_update() {
        let mut radar = Radar::new_with_config(RadarConfig {
            range_max: 100000.0,
            detection_angle_yaw: 180f32.to_radians(),
            detection_angle_pitch: 180f32.to_radians(),
            signal_strength: 1.0,
        });
        let radar_pose = Mat4::from_translation(vec3(0.0, 0.0, 0.0));
        let reflections = vec![
            (Mat4::from_translation(vec3(-5.0f32, 0.0, 0.0)), 1.0), // seen! 5m, behind the radar.
            (Mat4::from_translation(vec3(0.00001f32, 5.0, 0.0)), 1.0), // seen! way left.
            (Mat4::from_translation(vec3(0.00001f32, 0.0, 5.0)), 1.0), // Above the radar.
        ];

        radar.update_reflections(&radar_pose, &reflections);
        let obtained = radar.reflections();
        println!("Obtained: {obtained:?}");
        let expected = vec![(3.1415926f32, 0.0f32), (1.5707964, 0.0), (0.0, 1.5707964)];
        for (obtain, expect) in obtained.iter().zip(expected.iter()) {
            approx_equal!(obtain.yaw, expect.0, 0.001);
            approx_equal!(obtain.pitch, expect.1, 0.001);
        }
    }
    #[test]
    fn test_radar_north_update() {
        let mut radar = Radar::new_with_config(RadarConfig {
            range_max: 30.0,
            detection_angle_yaw: 60.0f32.to_radians(),
            detection_angle_pitch: 180f32.to_radians(),
            signal_strength: 1.0,
        });
        let reflections = vec![
            (Mat4::from_translation(vec3(-5.0f32, 10.0, 0.0)), 1.0), // seen! 7m
            (Mat4::from_translation(vec3(-7.5f32, 5.5, 0.0)), 1.0),  // seen! yaw 45 deg,
            (Mat4::from_translation(vec3(-7.5f32, 5.5, -2.5)), 1.0), // seen! yaw 45 deg, pitch negative
        ];
        let radar_pose =
            Mat4::from_translation(vec3(-5.0f32, 3.0, 0.0)) * Mat4::from_angle_z(cgmath::Deg(90.0));
        radar.update_reflections(&radar_pose, &reflections);
        use crate::util::cgmath::EuclideanNorm;
        let expected = vec![
            (0.0f32, 0.0f32, 7.0f32),
            (
                std::f32::consts::PI / 4.0,
                0.0f32,
                vec3(2.5, 2.5, 0.0).euclid_norm(),
            ),
            (
                std::f32::consts::PI / 4.0,
                -0.6154797,
                vec3(2.5, 2.5, 2.5).euclid_norm(),
            ),
        ];
        let obtained = radar.reflections();
        println!("Obtained: {obtained:?}");
        assert_eq!(obtained.len(), expected.len());
        for (obtain, expect) in obtained.iter().zip(expected.iter()) {
            approx_equal!(obtain.yaw, expect.0, 0.001);
            approx_equal!(obtain.pitch, expect.1, 0.001);
            approx_equal!(obtain.distance, expect.2, 0.001);
        }
    }
}
