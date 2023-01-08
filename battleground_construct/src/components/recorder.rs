use crate::components;
use crate::display;
use engine::prelude::*;
use serde::{Deserialize, Serialize};
use std::any::TypeId;

// use ciborium::{cbor, de::from_reader, ser::into_writer};
// use postcard::{from_bytes, to_vec};

pub type RecordingStorage = std::rc::Rc<std::cell::RefCell<Recording>>;

pub type ComponentState = Vec<(usize, Vec<u8>)>;

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
struct WorldState {
    component_lookup: std::collections::HashMap<String, usize>,
    states: Vec<(usize, ComponentState)>,
}

impl WorldState {
    fn capture_state<T: Component + Serialize + 'static>(world: &World) -> ComponentState {
        // const SIZE: usize = std::mem::size_of::<T>();
        world
            .component_iter::<T>()
            .map(|(e, c)| {
                (e.into(), {
                    // let mut encoded = Vec::with_capacity(std::mem::size_of::<T>());
                    // into_writer(&*c, &mut encoded).unwrap();
                    // encoded
                    postcard::to_vec::<_, 100>(&*c).unwrap().iter().copied().collect()
                    // v
                })
            })
            .collect()
    }
    fn add_component<T: Component + Serialize + 'static>(&mut self, name: &str, world: &World) {
        let c = self.component_lookup.len();
        let index = *self.component_lookup.get(name).get_or_insert(&c);
        self.states.push((*index, Self::capture_state::<T>(world)));
    }
}

#[derive(Debug, Clone, Default)]
pub struct Recording {
    states: Vec<WorldState>,
}
impl Recording {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn record(&mut self, world: &World) {
        // return;
        // println!("yay");
        // What do we actually need to record?
        // time
        // game status
        // poses
        // joint status
        // units, technically we can 'apply' the displays as an afterthought.
        // health
        // damage histories.

        // lets start simple.
        // Pose
        // Particle emitter.

        // and then see if this goes anywhere.
        let mut world_state = WorldState::default();
        // let clock_states = Self::capture_state::<components::clock::Clock>(world);
        world_state.add_component::<components::clock::Clock>("clock", world);
        world_state.add_component::<components::pose::Pose>("pose", world);
        world_state
            .add_component::<display::particle_emitter::ParticleEmitter>("particle_emitter", world);
        self.states.push(world_state);
    }
}

#[derive(Debug, Clone)]
pub struct Recorder {
    recording: RecordingStorage,
}

impl Recorder {
    pub fn new() -> Self {
        Recorder {
            recording: std::rc::Rc::new(std::cell::RefCell::new(Recording::new())),
        }
    }

    pub fn recording(&self) -> RecordingStorage {
        self.recording.clone()
    }
}
impl Component for Recorder {}
