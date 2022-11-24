pub type Mat4 = cgmath::Matrix4<f32>;
pub type Vec3 = cgmath::Vector3<f32>;

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

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Primitive {
    Cuboid(Cuboid),
    Sphere(Sphere),
    Cylinder(Cylinder),
}

impl std::hash::Hash for Primitive {
    fn hash<H>(&self, state: &mut H)
    where
        H: std::hash::Hasher,
    {
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
        }
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
pub struct ParticleEmitter {
    pub size: f32,
    pub color: Color,
}
impl Eq for ParticleEmitter {}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum EffectType {
    ParticleEmitter(ParticleEmitter),
}

#[derive(Debug, Copy, Clone)]
pub struct Effect {
    pub effect: EffectType,
    pub transform: Mat4,
}

pub trait Drawable {
    fn drawables(&self) -> Vec<Element> {
        vec![]
    }
    fn effects(&self) -> Vec<Effect> {
        vec![]
    }
}
