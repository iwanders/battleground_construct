use engine::prelude::*;

use super::unit::UnitId;
use crate::components::damage_hit::DamageHit;
use crate::components::impact::Impact;

#[derive(Debug, Clone)]
pub struct HitRecord {
    pub damage_hit: DamageHit,
    pub impact: Impact,
    pub source: Option<UnitId>,
    pub time: f32,
}

impl HitRecord {
    pub fn impact(&self) -> Impact {
        self.impact.clone()
    }
    pub fn source(&self) -> Option<UnitId> {
        self.source
    }
}

#[derive(Debug, Clone, Default)]
pub struct HitBy {
    hits: Vec<HitRecord>,
}

impl HitBy {
    pub fn new() -> Self {
        HitBy { hits: vec![] }
    }

    pub fn add_hit(
        &mut self,
        damage_hit: DamageHit,
        impact: Impact,
        source: Option<UnitId>,
        time: f32,
    ) {
        self.hits.push(HitRecord {
            damage_hit,
            impact,
            source,
            time,
        });
    }

    pub fn hits(&self) -> Vec<(f32, &Impact)> {
        self.hits
            .iter()
            .map(|v| (v.damage_hit.damage(), &v.impact))
            .collect::<_>()
    }
}
impl Component for HitBy {}

// ---------------------------------------------------

#[derive(Debug, Clone, Default)]
pub struct HitByHistory {
    hits: Vec<HitRecord>,
}

impl HitByHistory {
    pub fn new() -> Self {
        HitByHistory { hits: vec![] }
    }

    pub fn hits(&self) -> &[HitRecord] {
        &self.hits
    }
    pub fn add_hits(&mut self, hit_by: &HitBy) {
        self.hits.append(&mut hit_by.hits.clone())
    }
}
impl Component for HitByHistory {}
