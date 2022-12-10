use super::primitives::*;
use engine::prelude::*;

#[repr(C)]
#[derive(Debug, Clone, PartialEq)]
pub struct LineSegment {
    pub p0: [f32; 3],
    pub p1: [f32; 3],
    pub width: f32,
    pub color: Color,
}

impl LineSegment {
    pub fn into_le_bytes(self) -> [u8; std::mem::size_of::<LineSegment>()] {
        self.into()
    }
}

impl From<[u8; std::mem::size_of::<LineSegment>()]> for LineSegment {
    fn from(b: [u8; std::mem::size_of::<LineSegment>()]) -> Self {
        let read_f32 = |offset: usize| {
            let mut res = [0u8; 4];
            res[..].copy_from_slice(&b[offset * 4..(offset + 1) * 4]);
            f32::from_le_bytes(res)
        };
        LineSegment {
            p0: [read_f32(0), read_f32(1), read_f32(2)],
            p1: [read_f32(3), read_f32(4), read_f32(5)],
            width: read_f32(6),
            color: Color {
                r: b[6 * 4 + 0],
                g: b[6 * 4 + 1],
                b: b[6 * 4 + 2],
                a: b[6 * 4 + 3],
            },
        }
    }
}
impl From<LineSegment> for [u8; std::mem::size_of::<LineSegment>()] {
    fn from(l: LineSegment) -> [u8; std::mem::size_of::<LineSegment>()] {
        let mut res = [0u8; std::mem::size_of::<LineSegment>()];
        res[0..4].copy_from_slice(&l.p0[0].to_le_bytes());
        res[4..8].copy_from_slice(&l.p0[1].to_le_bytes());
        res[8..12].copy_from_slice(&l.p0[2].to_le_bytes());
        res[12..16].copy_from_slice(&l.p1[0].to_le_bytes());
        res[16..20].copy_from_slice(&l.p1[1].to_le_bytes());
        res[20..24].copy_from_slice(&l.p1[2].to_le_bytes());
        res[24..28].copy_from_slice(&l.width.to_le_bytes());
        res[28] = l.color.r;
        res[29] = l.color.g;
        res[30] = l.color.b;
        res[31] = l.color.a;
        res
    }
}

#[derive(Debug, Clone)]
pub struct DrawComponent {
    lines: Vec<LineSegment>,
}

impl DrawComponent {
    pub fn new() -> Self {
        DrawComponent { lines: vec![] }
    }
}
impl Component for DrawComponent {}

impl Drawable for DrawComponent {
    fn drawables(&self) -> Vec<Element> {
        let m = Mat4::from_translation(Vec3::new(0.0, 0.0, 0.0));
        println!("Drawing {} lines", self.lines.len());
        self.lines
            .iter()
            .map(|l| Element {
                transform: m,
                primitive: Primitive::Line(Line {
                    p0: (l.p0[0], l.p0[1], l.p0[2]),
                    p1: (l.p1[0], l.p1[1], l.p1[2]),
                    width: l.width,
                }),
                color: l.color,
            })
            .collect()
    }
}

#[derive(Debug, Clone)]
pub struct DrawModule {
    entity: EntityId,
}

impl DrawModule {
    pub fn new(entity: EntityId) -> Self {
        DrawModule { entity }
    }
}

use crate::components::vehicle_interface::{Register, RegisterMap, VehicleModule};

impl VehicleModule for DrawModule {
    fn get_registers(&self, _world: &World, registers: &mut RegisterMap) {
        registers.clear();
        registers.insert(0, Register::new_bytes("instructions"));
    }
    fn set_component(&self, world: &mut World, registers: &RegisterMap) {
        if let Some(mut draw_component) = world.component_mut::<DrawComponent>(self.entity) {
            draw_component.lines.clear();

            let instructions = registers
                .get(&0)
                .expect("register doesnt exist")
                .value_bytes()
                .expect("wrong value type");
            // println!("Instructions: {}", instructions.len());

            const INSTRUCTION_LEN: usize = std::mem::size_of::<LineSegment>();
            if instructions.len() % INSTRUCTION_LEN == 0 {
                let instruction_count = instructions.len() / INSTRUCTION_LEN;
                for i in 0..instruction_count {
                    let mut b = [0u8; INSTRUCTION_LEN];
                    b.copy_from_slice(
                        &instructions[i * INSTRUCTION_LEN..(i + 1) * INSTRUCTION_LEN],
                    );
                    draw_component.lines.push(b.into());
                }
            } else {
                println!("Instructions not complete.");
            }
        } else {
            println!("Could not find draw component");
        }
    }
}
