use super::primitives::*;
use engine::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Copy, Debug, Clone, PartialEq, Eq)]
pub struct ParticleEmitter {
    pub id: u64,
    pub particle_type: ParticleType,
    pub emitting: bool,
}

impl ParticleEmitter {
    pub fn bullet_trail(id: u64, size: f32, color: Color) -> Self {
        ParticleEmitter {
            id,
            particle_type: ParticleType::BulletTrail { size, color },
            emitting: true,
        }
    }
    pub fn bullet_impact(id: u64, size: f32, color: Color, velocity: Vec3) -> Self {
        ParticleEmitter {
            id,
            particle_type: ParticleType::BulletImpact {
                size,
                color,
                velocity,
            },
            emitting: false,
        }
    }
    pub fn muzzle_flash(id: u64, size: f32, color: Color) -> Self {
        ParticleEmitter {
            id,
            particle_type: ParticleType::MuzzleFlash { size, color },
            emitting: false,
        }
    }
    pub fn explosion(id: u64, radius: f32) -> Self {
        ParticleEmitter {
            id,
            particle_type: ParticleType::Explosion { radius },
            emitting: false,
        }
    }
}
impl Component for ParticleEmitter {}

impl Drawable for ParticleEmitter {
    fn effects(&self) -> Vec<Effect> {
        vec![Effect {
            id: self.id,
            effect: EffectType::ParticleEmitter {
                particle_type: self.particle_type,
                emitting: self.emitting,
            },
            transform: Mat4::from_translation(Vec3::new(0.0, 0.0, 0.0)),
        }]
    }
}
