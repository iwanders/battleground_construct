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

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Sphere {
    pub radius: f32,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Cylinder {
    pub radius: f32,
    pub height: f32,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Primitive {
    Cuboid(Cuboid),
    Sphere(Sphere),
    Cylinder(Cylinder),
}

impl std::hash::Hash for Primitive {
    fn hash<H>(&self, state: &mut H) where H: std::hash::Hasher {
        match *self {
            Primitive::Cuboid(cube) => {
                cube.width.to_bits().hash(state);
                cube.length.to_bits().hash(state);
                cube.height.to_bits().hash(state);
            },
            Primitive::Sphere(sphere) => {
                sphere.radius.to_bits().hash(state);
            },
            Primitive::Cylinder(cylinder) => {
                cylinder.radius.to_bits().hash(state);
                cylinder.height.to_bits().hash(state);
            },
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

#[derive(Debug, Copy, Clone)]
pub struct Element {
    pub primitive: Primitive,
    pub transform: Mat4,
    pub color: Color,
}

pub trait Drawable {
    fn drawables(&self) -> Vec<Element>;
}
