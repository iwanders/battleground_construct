use super::primitives::*;
use engine::prelude::*;

#[derive(Debug, Clone, Default)]
pub struct DebugLines {
    lines: Vec<(Line, Color)>,
}

impl DebugLines {
    pub fn new() -> Self {
        DebugLines::default()
    }
    pub fn straight(length: f32, width: f32, color: Color) -> Self {
        DebugLines {
            lines: vec![(
                Line {
                    p0: (0.0, 0.0, 0.0),
                    p1: (length, 0.0, 0.0),
                    width,
                },
                color,
            )],
        }
    }
    pub fn add_line(&mut self, line: Line, color: Color) {
        self.lines.push((line, color))
    }
}
impl Component for DebugLines {}

impl Drawable for DebugLines {
    fn drawables(&self) -> Vec<Element> {
        let m = Mat4::from_translation(Vec3::new(0.0, 0.0, 0.0));
        self.lines
            .iter()
            .map(|(l, c)| Element {
                transform: m,
                primitive: Primitive::Line(*l),
                material: (*c).into(),
            })
            .collect()
    }
}
