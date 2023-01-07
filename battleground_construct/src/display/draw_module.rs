use super::primitives::*;
use engine::prelude::*;

pub use battleground_unit_control::modules::draw::LineSegment;

#[derive(Debug, Clone, Default)]
pub struct DrawComponent {
    lines: Vec<LineSegment>,
}

impl DrawComponent {
    pub fn new() -> Self {
        DrawComponent::default()
    }
}
impl Component for DrawComponent {}

impl Drawable for DrawComponent {
    fn drawables(&self) -> Vec<Element> {
        let m = Mat4::from_translation(Vec3::new(0.0, 0.0, 0.0));
        self.lines
            .iter()
            .map(|l| Element {
                transform: m,
                primitive: Primitive::Line(Line {
                    p0: (l.p0[0], l.p0[1], l.p0[2]),
                    p1: (l.p1[0], l.p1[1], l.p1[2]),
                    width: l.width,
                }),
                material: Material::OverlayMaterial(
                    Color {
                        r: l.color[0],
                        g: l.color[1],
                        b: l.color[2],
                        a: l.color[3],
                    }
                    .into(),
                ),
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

use crate::components::unit_interface::{Register, RegisterMap, UnitModule};

impl UnitModule for DrawModule {
    fn get_registers(&self, _world: &World, registers: &mut RegisterMap) {
        registers.clear();
        registers.insert(
            battleground_unit_control::modules::draw::REG_DRAW_LINES,
            Register::new_bytes("instructions"),
        );
    }
    fn set_component(&self, world: &mut World, registers: &RegisterMap) {
        if let Some(mut draw_component) = world.component_mut::<DrawComponent>(self.entity) {
            draw_component.lines.clear();

            let instructions = registers
                .get(&battleground_unit_control::modules::draw::REG_DRAW_LINES)
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
