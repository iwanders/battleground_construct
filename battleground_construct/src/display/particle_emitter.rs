use super::primitives::*;
use engine::prelude::*;

#[derive(Copy, Debug, Clone)]
pub struct ParticleEmitter {
    pub size: f32,
    pub particle_color: Color,
}

impl Default for ParticleEmitter {
    fn default() -> Self {
        ParticleEmitter {
            size: 0.1,
            particle_color: Color {
                r: 255,
                g: 0,
                b: 0,
                a: 255,
            },
        }
    }
}

impl ParticleEmitter {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn from_scale_color(size: f32, color: Color) -> Self {
        let mut res = Self::default();
        res.size = size;
        res.particle_color = color;
        res
    }
}
impl Component for ParticleEmitter {}

impl Drawable for ParticleEmitter {
    fn effects(&self) -> Vec<Effect> {
        vec![Effect {
            effect: EffectType::ParticleEmitter(super::primitives::ParticleEmitter {
                size: 0.1,
                color: Color {
                    r: 255,
                    g: 0,
                    b: 255,
                    a: 128,
                },
            }),
            transform: Mat4::from_translation(Vec3::new(0.0, 0.0, 0.0)),
        }]
    }
}
