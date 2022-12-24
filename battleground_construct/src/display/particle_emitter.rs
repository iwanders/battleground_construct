use super::primitives::*;
use engine::prelude::*;

#[derive(Copy, Debug, Clone)]
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
