use super::primitives::*;
use engine::prelude::*;

#[derive(Copy, Debug, Clone)]
pub struct ParticleEmitter {
    pub entity: EntityId,
    pub particle_type: ParticleType,
}

impl ParticleEmitter {
    pub fn bullet_trail(entity: EntityId, size: f32, color: Color) -> Self {
        ParticleEmitter {
            entity,
            particle_type: ParticleType::BulletTrail { size, color },
        }
    }
}
impl Component for ParticleEmitter {}

impl Drawable for ParticleEmitter {
    fn effects(&self) -> Vec<Effect> {
        vec![Effect {
            id: (self.entity, 0),
            effect: EffectType::ParticleEmitter(self.particle_type),
            transform: Mat4::from_translation(Vec3::new(0.0, 0.0, 0.0)),
        }]
    }
}
