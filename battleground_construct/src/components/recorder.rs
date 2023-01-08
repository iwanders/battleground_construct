use crate::components;
use crate::display;
use engine::prelude::*;
use serde::{Deserialize, Serialize};

pub type RecordingStorage = std::rc::Rc<std::cell::RefCell<Recording>>;

pub type ComponentType = usize;

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
struct ComponentMap {
    // Name to component id.
    component_map: std::collections::HashMap<String, ComponentType>,
}
impl ComponentMap {
    pub fn get(&self, name: &str) -> Option<ComponentType> {
        self.component_map.get(name).copied()
    }

    pub fn insert(&mut self, name: &str) {
        let c = self.component_map.len();
        let v = self.component_map.get_mut(name);
        if v.is_none() {
            self.component_map.insert(name.to_owned(), c);
        }
    }

    pub fn components(&self) -> Vec<ComponentType> {
        self.component_map.values().copied().collect()
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
struct ComponentDelta {
    change: ComponentStates,
    removed: Vec<EntityId>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
struct ComponentStates {
    states: Vec<(EntityId, Vec<u8>)>,
}
impl ComponentStates {
    pub fn capture<T: Component + Serialize + 'static>(world: &World) -> ComponentStates {
        let states: Vec<(EntityId, Vec<u8>)> = world
            .component_iter::<T>()
            .map(|(e, c)| (e.into(), { bincode::serialize(&*c).unwrap() }))
            .collect();
        // states.sort();  // this is key, this way the delta can easily be created.
        ComponentStates { states }
    }

    pub fn to_hashmap(&self) -> std::collections::HashMap<EntityId, &[u8]> {
        self.states.iter().map(|(e, d)| (*e, &d[..])).collect()
    }

    pub fn delta(&self, new_states: &ComponentStates) -> ComponentDelta {
        let mut delta = ComponentDelta::default();
        // Cheap and dirty for now, convert to hashmap, compare.
        let old_map = self.to_hashmap();
        let new_map = new_states.to_hashmap();

        // First, determine removed. Removed is; was in old, not in new.
        for k_old in old_map.keys() {
            if !new_map.contains_key(k_old) {
                delta.removed.push(*k_old);
            }
        }

        // Next, after we've done removed, we need to add the changed / new components. This is
        // iterating over new, retrieve from old, ignore if equal.
        for (new_entity, new_data) in new_map.iter() {
            let equal = if let Some(old_data) = old_map.get(new_entity) {
                old_data == new_data
            } else {
                false
            };

            if !equal {
                delta.change.states.push((*new_entity, new_data.to_vec()));
            }
        }
        delta
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
struct WorldState {
    states: std::collections::HashMap<ComponentType, ComponentStates>,
}

impl WorldState {
    fn add_component<T: Component + Serialize + 'static>(
        &mut self,
        component_map: &ComponentMap,
        name: &str,
        world: &World,
    ) {
        // println!("{component_map:?}");
        let index = component_map
            .get(name)
            .expect(&format!("storing type {} not in map", name)); //.get_or_insert(&c);
        self.states
            .insert(index, ComponentStates::capture::<T>(world));
    }

    fn ensure_components(&mut self, component_map: &ComponentMap) {
        for index in component_map.components() {
            self.states.insert(index, Default::default());
        }
    }

    fn component_delta(&self, component: ComponentType, new_state: &WorldState) -> ComponentDelta {
        self.states
            .get(&component)
            .expect("component expected in old state")
            .delta(
                new_state
                    .states
                    .get(&component)
                    .expect("component expected in new state"),
            )
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
struct DeltaState {
    delta: std::collections::HashMap<ComponentType, ComponentDelta>,
}

impl DeltaState {
    pub fn new(old_state: &WorldState, new_state: &WorldState) -> Self {
        let mut delta = DeltaState::default();
        delta.capture(old_state, new_state);
        delta
    }

    fn capture(&mut self, old_state: &WorldState, new_state: &WorldState) {
        for component in new_state.states.keys() {
            let component_delta = old_state.component_delta(*component, new_state);
            self.delta.insert(*component, component_delta);
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
enum Capture {
    WorldState(WorldState),
    DeltaState(DeltaState),
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Recording {
    component_map: ComponentMap,
    states: Vec<Capture>,
    #[serde(skip)]
    previous_state: WorldState,
}
impl Recording {
    pub fn new() -> Self {
        let mut v = Recording::default();
        v.setup();
        v
    }

    fn setup(&mut self) {
        self.component_map.insert("clock");
        self.component_map.insert("pose");
        self.component_map.insert("particle_emitter");
        self.previous_state.ensure_components(&self.component_map);
        // println!("{:?}", self.component_map);
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
        let mut new_world_state = WorldState::default();
        // let clock_states = Self::capture_state::<components::clock::Clock>(world);
        new_world_state.add_component::<components::clock::Clock>(
            &self.component_map,
            "clock",
            world,
        );
        new_world_state.add_component::<components::pose::Pose>(&self.component_map, "pose", world);
        new_world_state.add_component::<display::particle_emitter::ParticleEmitter>(
            &self.component_map,
            "particle_emitter",
            world,
        );
        let delta = DeltaState::new(&self.previous_state, &new_world_state);
        self.states.push(Capture::DeltaState(delta));
        self.previous_state = new_world_state;
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
