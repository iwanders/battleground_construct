use super::primitives::*;
use engine::prelude::*;

#[repr(C)]
#[derive(Debug, Clone)]
pub struct LineSegment {
    p0: [f32; 3],
    p1: [f32; 3],
    width: f32,
    color: Color,
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

#[derive(Debug, Clone)]
pub struct DrawModule {
    entity: EntityId,
    lines: Vec<LineSegment>,
}

impl DrawModule {
    pub fn new(entity: EntityId) -> Self {
        DrawModule {
            entity,
            lines: vec![],
        }
    }
}
impl Component for DrawModule {}

impl Drawable for DrawModule {
    fn drawables(&self) -> Vec<Element> {
        let m = Mat4::from_translation(Vec3::new(0.0, 0.0, 0.0));
        self.lines
            .iter()
            .map(|l| Element {
                transform: m,
                primitive: Primitive::Line(Line {
                    p0: (l.p0[0], l.p0[1], l.p0[2]),
                    p1: (l.p0[3], l.p0[4], l.p0[5]),
                    width: l.width,
                }),
                color: l.color,
            })
            .collect()
    }
}

use crate::components::vehicle_interface::{Register, RegisterMap, VehicleModule};

impl VehicleModule for DrawModule {
    fn get_registers(&self, _world: &World, registers: &mut RegisterMap) {
        registers.clear();
        registers.insert(0, Register::new_bytes("instructions"));
    }
    fn set_component(&self, world: &mut World, registers: &RegisterMap) {
        if let Some(mut draw_module) = world.component_mut::<DrawModule>(self.entity) {
            draw_module.lines.clear();

            let instructions = registers
                .get(&0)
                .expect("register doesnt exist")
                .value_bytes()
                .expect("wrong value type");

            const INSTRUCTION_LEN: usize = std::mem::size_of::<LineSegment>();
            if instructions.len() % INSTRUCTION_LEN == 0 {
                let instruction_count = instructions.len() / INSTRUCTION_LEN;
                for i in 0..instruction_count {
                    let mut b = [0u8; INSTRUCTION_LEN];
                    b.copy_from_slice(
                        &instructions[i * INSTRUCTION_LEN..(i + 1) * INSTRUCTION_LEN],
                    );
                    draw_module.lines.push(b.into());
                }
            }
        }
    }
}
