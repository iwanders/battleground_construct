//! Drawing lines in the world.
//!
//! Lines are created by writing serialized [`LineSegment`] structs into the byte register.

/// Register accepts serialized [`LineSegment`] structs, multiple may be provided. Bytes value.
pub const REG_DRAW_LINES: u32 = 0;

#[repr(C)]
#[derive(Debug, Clone, PartialEq)]
/// Struct that is used to represent a line segment
pub struct LineSegment {
    /// The start point of the line.
    pub p0: [f32; 3],
    /// The end point of the line.
    pub p1: [f32; 3],
    /// the width of the line.
    pub width: f32,
    /// The color of the line in [r, g, b, a].
    pub color: [u8; 4],
}

impl LineSegment {
    /// Convenience function to serialize the [`LineSegment`] to bytes. The `From` trait is also
    /// implemented for both directions.
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
            color: [b[7 * 4], b[7 * 4 + 1], b[7 * 4 + 2], b[7 * 4 + 3]],
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
        res[28] = l.color[0];
        res[29] = l.color[1];
        res[30] = l.color[2];
        res[31] = l.color[3];
        res
    }
}
