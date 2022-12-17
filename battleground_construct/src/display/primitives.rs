pub type Mat4 = cgmath::Matrix4<f32>;
pub type Vec3 = cgmath::Vector3<f32>;
pub type Twist = crate::util::cgmath::Twist<f32>;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Cuboid {
    // Direction in x.
    pub width: f32,
    // Direction in y.
    pub length: f32,
    // Direction in z.
    pub height: f32,
}
impl Eq for Cuboid {}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Sphere {
    pub radius: f32,
}
impl Eq for Sphere {}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Cylinder {
    pub radius: f32,
    pub height: f32,
}
impl Eq for Cylinder {}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Line {
    pub p0: (f32, f32, f32),
    pub p1: (f32, f32, f32),
    pub width: f32,
}
impl Eq for Line {}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Primitive {
    Cuboid(Cuboid),
    Sphere(Sphere),
    Cylinder(Cylinder),
    Line(Line),
}

impl Primitive {
    pub fn to_draw_key(&self) -> u64 {
        use std::hash::Hash;
        use std::hash::Hasher;
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        let state = &mut hasher;
        match *self {
            Primitive::Cuboid(cube) => {
                0usize.hash(state);
                cube.width.to_bits().hash(state);
                cube.length.to_bits().hash(state);
                cube.height.to_bits().hash(state);
            }
            Primitive::Sphere(sphere) => {
                1usize.hash(state);
                sphere.radius.to_bits().hash(state);
            }
            Primitive::Cylinder(cylinder) => {
                2usize.hash(state);
                cylinder.radius.to_bits().hash(state);
                cylinder.height.to_bits().hash(state);
            }
            Primitive::Line(_line) => {
                // All lines hash the same.
                3usize.hash(state);
            }
        }
        // val
        hasher.finish()
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}
impl Color {
    pub const WHITE: Color = Color {
        r: 255,
        g: 255,
        b: 255,
        a: 255,
    };
    pub const RED: Color = Color {
        r: 255,
        g: 0,
        b: 0,
        a: 255,
    };
    pub const MAGENTA: Color = Color {
        r: 255,
        g: 0,
        b: 255,
        a: 255,
    };
    pub const BLUE: Color = Color {
        r: 0,
        g: 0,
        b: 255,
        a: 255,
    };
    pub const GREEN: Color = Color {
        r: 0,
        g: 255,
        b: 0,
        a: 255,
    };
}

#[derive(Debug, Copy, Clone)]
pub struct Element {
    pub primitive: Primitive,
    pub transform: Mat4,
    pub color: Color,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ParticleType {
    BulletTrail { size: f32, color: Color },
}
impl Eq for ParticleType {}

#[derive(Debug, Clone)]
pub enum EffectType {
    ParticleEmitter {
        particle_type: ParticleType,
        emitting: bool,
    },
    Deconstructor {
        elements: Vec<(Element, Twist)>,
        impacts: Vec<(Mat4, f32)>,
    },
}

pub type EffectId = (engine::EntityId, u64);

#[derive(Debug, Clone)]
pub struct Effect {
    /// Id to track this entity under, the EntityId is always associated to the entity that first
    /// created this effect. It does NOT tie this effect (nor its lifetime) to this particular
    /// entity.
    pub id: EffectId,
    /// Description of the effect itself.
    pub effect: EffectType,

    pub transform: Mat4,
}

pub trait Drawable {
    /// Drawable primitives for this frame. Nothing persistent between frames.
    fn drawables(&self) -> Vec<Element> {
        vec![]
    }
    /// Effects for this frame, these are longer lived and tracked by Id. Effects may be transfered
    /// between entities.
    fn effects(&self) -> Vec<Effect> {
        vec![]
    }
}
