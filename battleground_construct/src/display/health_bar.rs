use super::primitives::*;
use engine::prelude::*;

#[derive(Debug, Clone)]
pub struct HealthBar {
    /// Entity for which to track the health.
    entity: EntityId,
    full_length: f32,
    width: f32,
    height: f32,

    // Display state
    health: f32,
    color: Color,
}

impl HealthBar {
    pub fn new(entity: EntityId, full_length: f32) -> Self {
        HealthBar {
            entity,
            health: 1.0,
            full_length,
            width: 0.2,
            height: 0.05,
            color: Color::GREEN,
        }
    }

    pub fn health_entity(&self) -> EntityId {
        self.entity
    }

    pub fn set_health(&mut self, value: f32) {
        self.health = value;
        self.color = Color {
            r: (255.0 * (1.0 - self.health)) as u8,
            g: (255.0 * (self.health)) as u8,
            b: 0,
            a: 255,
        };
    }
}
impl Component for HealthBar {}

impl Drawable for HealthBar {
    fn drawables(&self) -> Vec<Element> {
        let m = Mat4::from_translation(Vec3::new(-self.full_length / 2.0, 0.0, 0.0));
        vec![Element {
            transform: m,
            primitive: Primitive::ExtrudedRectangle(ExtrudedRectangle {
                length: self.full_length * self.health,
                width: self.width,
                height: self.height,
            }),
            material: Material::FlatMaterial(self.color.into()),
        }]
    }
}
