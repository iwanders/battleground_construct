pub type Mat4 = cgmath::Matrix4<f32>;
pub type Vec3 = cgmath::Vector3<f32>;

#[derive(Debug, Copy, Clone)]
pub struct Cuboid {
    // Direction in x.
    pub width: f32,
    // Direction in y.
    pub length: f32,
    // Direction in z.
    pub height: f32,
}

#[derive(Debug, Copy, Clone)]
pub struct Sphere {
    pub radius: f32,
}

#[derive(Debug, Copy, Clone)]
pub struct Cylinder {
    pub radius: f32,
    pub height: f32,
}

#[derive(Debug, Copy, Clone)]
pub enum Primitive {
    Cuboid(Cuboid),
    Sphere(Sphere),
    Cylinder(Cylinder),
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
