use battleground_unit_control::log;
use battleground_unit_control::modules::{clock::*, controller::*, revolute::*, unit::*};
use battleground_unit_control::{Interface, UnitControl};
// Module constants live in common and their respective units.
use battleground_unit_control::modules::draw::LineSegment;
use battleground_unit_control::units::{common, UnitType};
use cgmath_util;
use cgmath_util::prelude::*;
use cgmath_util::vec3;

type Twist = cgmath_util::Twist<f32>;

const GREEN: [u8; 4] = [0, 255, 0, 255];
const BLUE: [u8; 4] = [0, 0, 255, 255];
const RED: [u8; 4] = [255, 0, 0, 255];
const TRANSPARENT_MAGENTA: [u8; 4] = [255, 0, 255, 64];

/// Our example controller!
pub struct UnitControlExample {
    last_print: f32,
}
impl Default for UnitControlExample {
    fn default() -> Self {
        Self {
            last_print: -10000.0,
        }
    }
}

pub struct JointP {
    k_p: f32,
    set_point: f32,
    revolute: u32,
    position: f32,
}
impl JointP {
    pub fn new(revolute: u32) -> Self {
        JointP {
            k_p: 0.5,
            set_point: 0.0,
            revolute,
            position: 0.0,
        }
    }
    pub fn set_point(&mut self, v: f32) {
        self.set_point = v;
    }

    pub fn position(&self) -> f32 {
        self.position
    }

    pub fn poll(
        &mut self,
        interface: &mut dyn Interface,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.position = interface.get_f32(self.revolute, REG_REVOLUTE_POSITION)?;
        Ok(())
    }

    pub fn update(
        &mut self,
        interface: &mut dyn Interface,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.poll(interface)?;
        let the_error = self.set_point - self.position;
        let control = self.k_p * the_error;
        interface.set_f32(self.revolute, REG_REVOLUTE_VELOCITY_CMD, control)?;
        // log::info!("Setting control to : {control}", );
        Ok(())
    }
}

impl UnitControl for UnitControlExample {
    /// This function gets called periodically to control our unit.
    fn update(&mut self, interface: &mut dyn Interface) -> Result<(), Box<dyn std::error::Error>> {
        draw_clear(interface)?;

        // This gets the current time.
        let t = interface.get_f32(common::MODULE_CLOCK, REG_CLOCK_ELAPSED)?;
        let PI_2 = std::f32::consts::PI / 2.0;

        let l0 = 1.0;
        let l1 = 1.0;
        let l2 = 1.0;

        let rev_base = 1;
        let rev_arm = 2;
        let rev_elbow = 3;

        let mut c_0 = JointP::new(rev_base);
        c_0.poll(interface)?;
        let mut c_1 = JointP::new(rev_arm);
        c_1.poll(interface)?;
        let mut c_2 = JointP::new(rev_elbow);
        c_2.poll(interface)?;
        // c_1.set_point(PI_2);

        // c_0.update(interface);
        // c_1.update(interface);
        // c_2.update(interface);
        // return Ok(());

        let T100 = Twist::new(vec3(0.0, 0.0, 0.0), vec3(0.0, 0.0, 1.0));
        let T210 = Twist::new(vec3(0.0, l0, 0.0), vec3(1.0, 0.0, 0.0));
        let T310 = Twist::new(vec3(0.0, l0 + l1, 0.0), vec3(1.0, 0.0, 0.0));

        let H100 = vec3(0.0, 0.0, l0).to_h();
        let H200 = vec3(0.0, 0.0, l0 + l1).to_h();
        let H300 = vec3(0.0, 0.0, l0 + l1 + l2).to_h();

        let H1_0 = (T100 * c_0.position()).exp() * H100;
        let H2_0 = (T100 * c_0.position()).exp() * (T210 * c_1.position()).exp() * H200;
        let H3_0 = (T100 * c_0.position()).exp()
            * (T210 * c_1.position()).exp()
            * (T310 * c_2.position()).exp()
            * H300;

        // This can be used to retrieve the unit type.
        let unit_type = interface.get_i32(common::MODULE_UNIT, REG_UNIT_UNIT_TYPE)?;
        let unit_type: UnitType = (unit_type as u32).try_into()?;

        draw_frame(interface, H1_0)?;
        draw_frame(interface, H2_0)?;
        draw_frame(interface, H3_0)?;

        Ok(())
    }
}

fn dump_registers(interface: &dyn Interface) -> Result<(), Box<dyn std::error::Error>> {
    use battleground_unit_control::RegisterType;
    for module_index in interface.modules()? {
        log::info!("module_name: {}", interface.module_name(module_index)?);
        let registers = interface.registers(module_index)?;
        for register_index in registers {
            let name = interface.register_name(module_index, register_index)?;
            let register_type = interface.register_type(module_index, register_index)?;
            match register_type {
                RegisterType::I32 => {
                    let v = interface.get_i32(module_index, register_index)?;
                    log::info!("    {module_index:0>8x}:{register_index:0>8x} (i32) {name}: {v}");
                }
                RegisterType::F32 => {
                    let v = interface.get_f32(module_index, register_index)?;
                    log::info!("    {module_index:0>8x}:{register_index:0>8x} (f32) {name}: {v}");
                }
                RegisterType::Bytes => {
                    let len = interface.get_bytes_len(module_index, register_index)?;
                    let mut read_v = vec![0; len];
                    interface.get_bytes(module_index, register_index, &mut read_v)?;
                    log::info!("    {module_index:0>8x}:{register_index:0>8x} (bytes {len}) {name}: {read_v:?}");
                }
            }
        }
    }
    Ok(())
}

fn draw_clear(interface: &mut dyn Interface) -> Result<(), Box<dyn std::error::Error>> {
    interface.set_bytes(
        common::MODULE_DRAW,
        battleground_unit_control::modules::draw::REG_DRAW_LINES,
        &[],
    )?;
    Ok(())
}

fn draw_line(
    interface: &mut dyn Interface,
    l: LineSegment,
) -> Result<(), Box<dyn std::error::Error>> {
    let len = interface.get_bytes_len(
        common::MODULE_DRAW,
        battleground_unit_control::modules::draw::REG_DRAW_LINES,
    )?;
    let mut read_v = vec![0; len];
    interface.get_bytes(
        common::MODULE_DRAW,
        battleground_unit_control::modules::draw::REG_DRAW_LINES,
        &mut read_v,
    )?;
    read_v.extend(l.into_le_bytes());

    let v = interface.set_bytes(
        common::MODULE_DRAW,
        battleground_unit_control::modules::draw::REG_DRAW_LINES,
        &read_v,
    )?;
    Ok(())
}

fn draw_frame(
    interface: &mut dyn Interface,
    h: cgmath_util::Mat4,
) -> Result<(), Box<dyn std::error::Error>> {
    let origin = vec3(0.0, 0.0, 0.0).to_h();
    let h_origin = h * origin;
    let r = 0.25;
    let w = 0.1;
    let x0 = vec3(r, 0.0, 0.0).to_h();
    let x0_origin = h * x0;
    let x1 = vec3(0.0, r, 0.0).to_h();
    let x1_origin = h * x1;
    let x2 = vec3(0.0, 0.0, r).to_h();
    let x2_origin = h * x2;
    draw_line(
        interface,
        LineSegment {
            p0: x0_origin.to_translation().into(),
            p1: h_origin.to_translation().into(),
            width: w,
            color: RED,
        },
    )?;
    draw_line(
        interface,
        LineSegment {
            p0: x1_origin.to_translation().into(),
            p1: h_origin.to_translation().into(),
            width: w,
            color: GREEN,
        },
    )?;
    draw_line(
        interface,
        LineSegment {
            p0: x2_origin.to_translation().into(),
            p1: h_origin.to_translation().into(),
            width: w,
            color: BLUE,
        },
    )?;
    Ok(())
}

#[no_mangle]
#[cfg(target_arch = "wasm32")]
pub fn create_unit_control() -> Box<dyn UnitControl> {
    Box::new(UnitControlExample::default())
}
