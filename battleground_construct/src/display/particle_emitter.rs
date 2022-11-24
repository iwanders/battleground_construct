use super::primitives::*;
use engine::prelude::*;

#[derive(Copy, Debug, Clone)]
pub struct ParticleEmitter {
    pub entity: EntityId,
    pub size: f32,
    pub particle_color: Color,
}



impl ParticleEmitter {


    pub fn from_scale_color(entity: EntityId, size: f32, color: Color) -> Self {
        ParticleEmitter{
            entity,
            size,
            particle_color: color,
        }
    }
}
impl Component for ParticleEmitter {}

impl Drawable for ParticleEmitter {
    fn effects(&self) -> Vec<Effect> {
        vec![Effect {
            id: (self.entity, 0),
            effect: EffectType::ParticleEmitter(super::primitives::ParticleEmitter {
                size: 0.1,
                color: Color {
                    r: 255,
                    g: 0,
                    b: 255,
                    a: 128,
                },
            }),
            // transform: Mat4::from_translation(Vec3::new(0.0, 0.0, 0.0)),
        }]
    }
}
