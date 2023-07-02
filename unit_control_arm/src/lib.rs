#![allow(non_snake_case)]
// #![allow(unused_imports)]
// #![allow(dead_code)]
// #![allow(unused_variables)]
// use battleground_unit_control::log;
use battleground_unit_control::modules::{clock::*, revolute::*};
use battleground_unit_control::{Interface, UnitControl};
// Module constants live in common and their respective units.
use battleground_unit_control::modules::draw::LineSegment;
use battleground_unit_control::units::common;
// use cgmath_util;
use cgmath_util::prelude::*;
use cgmath_util::vec3;

type Twist = cgmath_util::Twist<f32>;

const GREEN: [u8; 4] = [0, 255, 0, 255];
const BLUE: [u8; 4] = [0, 0, 255, 255];
const RED: [u8; 4] = [255, 0, 0, 255];
const TRANSPARENT_MAGENTA: [u8; 4] = [255, 0, 255, 64];

/// Our example controller!
#[derive(Default)]
pub struct UnitControlExample {}

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

    pub fn set_velocity(
        &self,
        interface: &mut dyn Interface,
        v: f32,
    ) -> Result<(), Box<dyn std::error::Error>> {
        interface.set_f32(self.revolute, REG_REVOLUTE_VELOCITY_CMD, v)?;
        Ok(())
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
        self.set_velocity(interface, control)?;
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

        draw_frame(interface, H1_0)?;
        draw_frame(interface, H2_0)?;
        draw_frame(interface, H3_0)?;

        draw_trajectory(interface, t)?;

        // Time to calculate the control law.
        let setpoint = figure_eight_trajectory(t);
        let error = setpoint - H3_0.w.truncate();
        let xd = 2.5 * error;

        // Unit twists in their body coordinates.
        let T100u = Twist::new(vec3(0.0, 0.0, 0.0), vec3(0.0, 0.0, 1.0));
        let T211u = Twist::new(vec3(0.0, 0.0, 0.0), vec3(1.0, 0.0, 0.0));
        let T311u = Twist::new(vec3(0.0, 0.0, 0.0), vec3(1.0, 0.0, 0.0));

        let T210_ut = H1_0.to_adjoint() * T211u;
        let T320_ut = H2_0.to_adjoint() * T311u;
        // log::info!("T210_ut: {T210_ut:?}");

        // ugh, now we need a 6 x 3 matrix :/
        // but an adjoint can be multiplied by a twist, and we only care about the position.
        let adj_neg_h3_0 = (-H3_0.w.truncate()).to_h().to_adjoint();
        let c0 = adj_neg_h3_0 * T100u;
        let c1 = adj_neg_h3_0 * T210_ut;
        let c2 = adj_neg_h3_0 * T320_ut;
        let J_part = cgmath::Matrix3::<f32>::from_cols(c0.v, c1.v, c2.v);
        use cgmath::SquareMatrix;
        let pseudo = J_part.transpose() * (J_part * J_part.transpose()).invert().unwrap();
        let qd = pseudo * xd;
        c_0.set_velocity(interface, qd.x)?;
        c_1.set_velocity(interface, qd.y)?;
        c_2.set_velocity(interface, qd.z)?;

        Ok(())
    }
}

fn draw_clear(interface: &mut dyn Interface) -> Result<(), Box<dyn std::error::Error>> {
    interface.set_bytes(
        common::MODULE_DRAW,
        battleground_unit_control::modules::draw::REG_DRAW_LINES,
        &[],
    )?;
    Ok(())
}

const RATE: f32 = 0.4;
fn figure_eight_trajectory(t: f32) -> cgmath::Vector3<f32> {
    vec3(
        0.8 * (t * RATE).cos(),
        -1.5,
        1.5 + 0.6 * (2.0 * (t * RATE)).sin(),
    )
}
fn draw_trajectory(
    interface: &mut dyn Interface,
    t: f32,
) -> Result<(), Box<dyn std::error::Error>> {
    let l = std::f32::consts::PI * 2.0 / RATE;
    let dl = l / 100.0;
    for k in 0..101 {
        let x0 = k as f32 * dl;
        let x1 = (k + 1) as f32 * dl;
        let p0 = figure_eight_trajectory(x0);
        let p1 = figure_eight_trajectory(x1);
        draw_line(
            interface,
            LineSegment {
                p0: p0.into(),
                p1: p1.into(),
                width: 0.01,
                color: TRANSPARENT_MAGENTA,
            },
        )?;
    }
    let p0 = figure_eight_trajectory(t);
    draw_line(
        interface,
        LineSegment {
            p0: p0.into(),
            p1: (p0 + vec3(0.00, 0.00, -0.01)).into(),
            width: 0.1,
            color: GREEN,
        },
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

    interface.set_bytes(
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
    let w = 0.01;
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
