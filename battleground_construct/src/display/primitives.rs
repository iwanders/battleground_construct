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
pub struct Cone {
    pub radius: f32,
    pub height: f32,
}
impl Eq for Cone {}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Line {
    pub p0: (f32, f32, f32),
    pub p1: (f32, f32, f32),
    pub width: f32,
}
impl Eq for Line {}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Circle {
    pub radius: f32,
    pub subdivisions: u32,
}
impl Eq for Circle {}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Primitive {
    Cuboid(Cuboid),
    Circle(Circle),
    Sphere(Sphere),
    Cylinder(Cylinder),
    Line(Line),
    Cone(Cone),
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
    pub const BLACK: Color = Color {
        r: 0,
        g: 0,
        b: 0,
        a: 0,
    };
    pub const GREY: Color = Color {
        r: 128,
        g: 128,
        b: 128,
        a: 255,
    };
    pub fn rgb(r: u8, g: u8, b: u8) -> Self {
        Color { r, g, b, a: 255 }
    }
}

impl From<(u8, u8, u8)> for Color {
    fn from(v: (u8, u8, u8)) -> Self {
        Color {
            r: v.0,
            g: v.1,
            b: v.2,
            a: 255,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct FlatMaterial {
    pub color: Color,
    pub emissive: Color,
    pub is_transparent: bool,
    pub is_emissive: bool,
}

impl Default for FlatMaterial {
    fn default() -> Self {
        FlatMaterial {
            color: Color::MAGENTA,
            emissive: Color::BLACK,
            is_transparent: false,
            is_emissive: false,
        }
    }
}

impl From<Color> for Material {
    fn from(color: Color) -> Material {
        Material::FlatMaterial(FlatMaterial {
            color,
            ..Default::default()
        })
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Material {
    FlatMaterial(FlatMaterial),
    TeamMaterial,
}

#[derive(Debug, Copy, Clone)]
pub struct Element {
    pub primitive: Primitive,
    pub transform: Mat4,
    pub material: Material,
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
        elements: Vec<(Element, /* entity */ Twist, /* entity */ Mat4)>,
        impacts: Vec<(Mat4, f32)>,
    },
}

#[derive(Debug, Clone)]
pub struct Effect {
    /// Id to track this effect under, the effect may be transfered between entities, the renderer
    /// should consider the same id as the same effect and carry over any state.
    pub id: u64,
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
