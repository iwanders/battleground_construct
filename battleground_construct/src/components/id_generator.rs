use engine::prelude::*;

#[derive(Debug, Clone, Default)]
pub struct IdGenerator {
    id: u64,
}

impl IdGenerator {
    pub fn new() -> Self {
        IdGenerator { id: 0 }
    }

    pub fn generate(&mut self) -> u64 {
        self.id += 1;
        self.id
    }
}
impl Component for IdGenerator {}

pub fn generate_id(world: &mut World) -> u64 {
    let (_entity, mut id_generator) = world
        .component_iter_mut::<IdGenerator>()
        .next()
        .expect("should have a generator, are default components added?");
    id_generator.generate()
}
