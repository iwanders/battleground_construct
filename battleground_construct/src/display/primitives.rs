pub use crate::util::cgmath::{Mat4, Vec3};
pub type Twist = crate::util::cgmath::Twist<f32>;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Copy, Clone, PartialEq)]
pub struct Cuboid {
    // Direction in x.
    pub length: f32,
    // Direction in y.
    pub width: f32,
    // Direction in z.
    pub height: f32,
}
impl Eq for Cuboid {}

#[derive(Deserialize, Serialize, Debug, Copy, Clone, PartialEq)]
pub struct Sphere {
    pub radius: f32,
}
impl Eq for Sphere {}

#[derive(Deserialize, Serialize, Debug, Copy, Clone, PartialEq)]
pub struct Cylinder {
    pub radius: f32,
    pub height: f32,
}
impl Eq for Cylinder {}

#[derive(Deserialize, Serialize, Debug, Copy, Clone, PartialEq)]
pub struct Cone {
    pub radius: f32,
    pub height: f32,
}
impl Eq for Cone {}

#[derive(Deserialize, Serialize, Debug, Copy, Clone, PartialEq)]
pub struct Line {
    pub p0: (f32, f32, f32),
    pub p1: (f32, f32, f32),
    pub width: f32,
}
impl Eq for Line {}

#[derive(Deserialize, Serialize, Debug, Copy, Clone, PartialEq)]
pub struct Circle {
    pub radius: f32,
}
impl Eq for Circle {}

/// This type grows the cube from one side.
#[derive(Deserialize, Serialize, Debug, Copy, Clone, PartialEq)]
pub struct ExtrudedRectangle {
    // Length of the extrusion
    pub length: f32,
    // Width in y.
    pub width: f32,
    // Height in z.
    pub height: f32,
}
impl Eq for ExtrudedRectangle {}

#[derive(Deserialize, Serialize, Debug, Copy, Clone, PartialEq, Eq)]
pub enum Primitive {
    Cuboid(Cuboid),
    Circle(Circle),
    Sphere(Sphere),
    Cylinder(Cylinder),
    Line(Line),
    Cone(Cone),
    ExtrudedRectangle(ExtrudedRectangle),
}

#[derive(Deserialize, Serialize, Debug, Copy, Clone, Eq, PartialEq)]
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
    pub fn saturating_add(&self, rhs: &Color) -> Self {
        Color {
            r: self.r.saturating_add(rhs.r),
            g: self.g.saturating_add(rhs.g),
            b: self.b.saturating_add(rhs.b),
            a: self.a.saturating_add(rhs.a),
        }
    }
    pub fn modified_alpha(&self, new_alpha: u8) -> Self {
        Color {
            r: self.r,
            g: self.g,
            b: self.b,
            a: new_alpha,
        }
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

#[derive(Deserialize, Serialize, Debug, Copy, Clone, PartialEq, Eq)]
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

impl From<Color> for FlatMaterial {
    fn from(color: Color) -> FlatMaterial {
        FlatMaterial {
            color,
            ..Default::default()
        }
    }
}

impl From<Color> for Material {
    fn from(color: Color) -> Material {
        Material::FlatMaterial(color.into())
    }
}

#[derive(Deserialize, Serialize, Debug, Copy, Clone, PartialEq, Eq)]
pub struct FenceMaterial {
    pub color: Color,
}

impl Default for FenceMaterial {
    fn default() -> Self {
        FenceMaterial {
            color: Color::MAGENTA,
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Copy, Clone, PartialEq, Eq)]
pub struct OverlayMaterial {
    pub color: Color,
    pub behind_color: Color,
}

impl Default for OverlayMaterial {
    fn default() -> Self {
        OverlayMaterial {
            color: Color::MAGENTA,
            behind_color: Color::MAGENTA,
        }
    }
}

impl From<Color> for OverlayMaterial {
    fn from(color: Color) -> OverlayMaterial {
        OverlayMaterial {
            color,
            behind_color: color.modified_alpha(color.a / 2),
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Copy, Clone, PartialEq, Eq)]
pub enum Material {
    FlatMaterial(FlatMaterial),
    FenceMaterial(FenceMaterial),
    OverlayMaterial(OverlayMaterial),
}

#[derive(Deserialize, Serialize, Debug, Copy, Clone)]
pub struct Element {
    pub primitive: Primitive,
    pub transform: Mat4,
    pub material: Material,
}

#[derive(Deserialize, Serialize, Debug, Copy, Clone, PartialEq)]
pub enum ParticleType {
    BulletTrail {
        size: f32,
        color: Color,
    },
    BulletImpact {
        size: f32,
        color: Color,
        velocity: Vec3,
    },
    Explosion {
        /// Radius of the explosion, not of particles.
        radius: f32,
    },
    MuzzleFlash {
        size: f32,
        color: Color,
    },
    Firework {
        /// Radius of firework, not of particles.
        radius: f32,
        color: Color,
    },
}
impl Eq for ParticleType {}

#[derive(Deserialize, Serialize, Debug, Clone)]
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

#[derive(Deserialize, Serialize, Debug, Clone)]
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
