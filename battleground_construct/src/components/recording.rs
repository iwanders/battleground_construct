use crate::components;
use crate::display;
use engine::prelude::*;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use miniz_oxide::deflate::compress_to_vec;
use miniz_oxide::inflate::decompress_to_vec;

/// Marker to denote a unit has been reconstructed.
pub struct PlaybackUnitCreatedMarker;
impl Component for PlaybackUnitCreatedMarker {}

/// Marker to denote a unit has been deconstructed.
pub struct PlaybackUnitDestroyedMarker;
impl Component for PlaybackUnitDestroyedMarker {}

/// Marker to denote playback is finished, this is added to the clock entity.
pub struct PlaybackFinishedMarker;
impl Component for PlaybackFinishedMarker {}

/// Pointer to internal data storage.
pub type RecordStorage = std::rc::Rc<std::cell::RefCell<Record>>;

/// Type ids aren't stable, so a map from names to this is kept.
pub type ComponentType = usize;

/// Map of components, from name to type id. Map is serialized at the beginning so the names
/// are stable, id's aren't.
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
struct ComponentMap {
    /// Name to component id.
    component_map: std::collections::HashMap<String, ComponentType>,
}
impl ComponentMap {
    pub fn insert(&mut self, name: &str) -> ComponentType {
        let c = self.component_map.len();
        let v = self.component_map.get_mut(name);
        if v.is_none() {
            self.component_map.insert(name.to_owned(), c);
        }
        *self.component_map.get(name).unwrap()
    }

    pub fn components(&self) -> Vec<ComponentType> {
        self.component_map.values().copied().collect()
    }

    pub fn get(&self, name: &str) -> Option<ComponentType> {
        self.component_map.get(name).copied()
    }
}

/// Delta for a particular component, this can be applied to a World.
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
struct ComponentDelta {
    /// Changed components, also holds new components (they're changed afterall).
    change: ComponentStates,

    /// Components to be removed.
    removed: Vec<EntityId>,
}

impl ComponentDelta {
    /// Apply this delta to a world.
    fn apply<T: Component + DeserializeOwned + 'static>(&self, world: &mut World) {
        for to_be_removed in self.removed.iter() {
            world.remove_component::<T>(*to_be_removed);
        }
        for (entity, data) in self.change.states() {
            let in_place_update =
                if let Some(mut current_component) = world.component_mut::<T>(*entity) {
                    *current_component = bincode::deserialize::<T>(&data[..]).unwrap();
                    true
                } else {
                    false
                };
            if !in_place_update {
                world.add_component(*entity, bincode::deserialize::<T>(&data[..]).unwrap());
            }
        }
    }

    fn apply_dry(&self, states: &mut ComponentStates) {
        let d = states.raw_mut();
        // Assume remove is shorter, use that as outer loop.
        for to_be_removed in self.removed.iter() {
            if let Some(present) = d.iter().position(|r| r.0 == *to_be_removed) {
                // println!("Removing {present:?}");
                d.swap_remove(present);
                break; // found this entry to be removed, don't iterate over the remainder.
            }
        }

        // Next, iterate over change.
        for to_change in self.change.states().iter() {
            if let Some(present) = d.iter().position(|r| r.0 == to_change.0) {
                // already had this, change it.
                // println!("Updating entry for {present:?}");
                d[present].1 = to_change.1.clone();
            } else {
                // println!("new entry for {to_change:?}");
                // new entry
                d.push(to_change.clone());
            }
        }
    }

    pub fn sum_bytes(&self) -> usize {
        self.change.sum_bytes() + self.removed.len() * 8
    }
}

/// Serialized component states for a particular component type.
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
struct ComponentStates {
    states: Vec<(EntityId, Vec<u8>)>,
}
impl ComponentStates {
    pub fn raw_mut(&mut self) -> &mut Vec<(EntityId, Vec<u8>)> {
        &mut self.states
    }

    /// Capture the component states for this component type.
    pub fn capture<T: Component + Serialize + 'static>(world: &World) -> ComponentStates {
        let states: Vec<(EntityId, Vec<u8>)> = world
            .component_iter::<T>()
            .map(|(e, c)| (e, { bincode::serialize(&*c).unwrap() }))
            .collect();
        // states.sort();  // this is key, this way the delta can easily be created.
        ComponentStates { states }
    }

    /// Retrieval method for the hashmap
    pub fn to_hashmap(&self) -> std::collections::HashMap<EntityId, &[u8]> {
        self.states.iter().map(|(e, d)| (*e, &d[..])).collect()
    }

    pub fn sum_bytes(&self) -> usize {
        self.states.iter().map(|(_, p)| 8 + p.len()).sum()
    }

    /// Given this ComponentStates and the new_states, calculate the delta.
    pub fn delta(&self, new_states: &ComponentStates) -> ComponentDelta {
        let mut delta = ComponentDelta::default();

        // We could make this faster, sorting on the capture and then advancing the two indices
        // through the sorted vectors.

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

    /// Retrieve the component states.
    pub fn states(&self) -> &[(EntityId, Vec<u8>)] {
        &self.states[..]
    }
}

/// A full representation of the world, holding the component states for all components.
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
struct WorldState {
    states: std::collections::HashMap<ComponentType, ComponentStates>,
}

impl WorldState {
    /// Add a component's full state to the world state.s
    fn add_component<T: Component + Serialize + 'static>(
        &mut self,
        index: ComponentType,
        world: &World,
    ) {
        self.states
            .insert(index, ComponentStates::capture::<T>(world));
    }

    /// Ensure empty vectors exist for all entries in the component map.
    fn ensure_components(&mut self, component_map: &ComponentMap) {
        for index in component_map.components() {
            self.states.insert(index, Default::default());
        }
    }

    /// Determine the component delta, given this state, a component type and a new state.
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

    pub fn component_states(&self, component: ComponentType) -> Option<&ComponentStates> {
        self.states.get(&component)
    }
    pub fn component_states_mut(
        &mut self,
        component: ComponentType,
    ) -> Option<&mut ComponentStates> {
        self.states.get_mut(&component)
    }
}

/// Delta state for the entire world.
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
struct DeltaState {
    delta: std::collections::HashMap<ComponentType, ComponentDelta>,
}

impl DeltaState {
    /// Create a new delta state given the old state and the new state.
    pub fn new(old_state: &WorldState, new_state: &WorldState) -> Self {
        let mut delta = DeltaState::default();
        delta.capture(old_state, new_state);
        delta
    }

    pub fn component_delta(&self, component: ComponentType) -> Option<&ComponentDelta> {
        self.delta.get(&component)
    }

    /// Internal worker to actually capture the states.
    fn capture(&mut self, old_state: &WorldState, new_state: &WorldState) {
        for component in new_state.states.keys() {
            let component_delta = old_state.component_delta(*component, new_state);
            self.delta.insert(*component, component_delta);
        }
    }

    /// Apply this delta to the provided world for the specified component type.
    fn apply<T: Component + DeserializeOwned + 'static>(
        &self,
        component: ComponentType,
        world: &mut World,
    ) {
        if let Some(delta) = self.delta.get(&component) {
            delta.apply::<T>(world);
        }
    }

    fn apply_dry(&self, world_state: &mut WorldState) {
        for (k, delta) in self.delta.iter() {
            if let Some(ref mut v) = world_state.component_states_mut(*k) {
                delta.apply_dry(v);
            }
        }
    }

    /// Retrieve a compressed version of this delta state.
    fn compressed(&self) -> Vec<u8> {
        let bytes = bincode::serialize(self).unwrap();
        compress_to_vec(&bytes, 6)
    }

    /// Create a delta state from a sequence of bytes.
    fn uncompress(bytes: &[u8]) -> DeltaState {
        let decompressed = decompress_to_vec(bytes).expect("Failed to decompress!");
        bincode::deserialize(&decompressed).unwrap()
    }
}

#[allow(clippy::enum_variant_names)]
#[derive(Debug, Clone, Deserialize, Serialize)]
/// Capture frame types.
enum Capture {
    /// Full representation of the world.
    WorldState(WorldState),
    /// Delta, to be applied to the previous world.
    DeltaState(DeltaState),
    /// Can be uncompressed into a DeltaState.
    ZippedDeltaState(Vec<u8>),
}

/// Helper function for type erasure to record.
type RecordFunction = Box<dyn Fn(&mut WorldState, &World)>;
/// Helper function for type erasure to play back.
type PlayFunction = Box<dyn Fn(&DeltaState, &mut World)>;

struct TypeHandler {
    record: RecordFunction,
    play: PlayFunction,
}

#[derive(Default, Deserialize, Serialize)]
/// The storage of the recording or playback.
pub struct Record {
    component_map: ComponentMap,
    states: Vec<Capture>,
    max_time: Option<f32>,
    #[serde(skip)]
    current_state: WorldState,
    #[serde(skip)]
    playback_index: usize,
    #[serde(skip)]
    helpers: std::collections::HashMap<ComponentType, TypeHandler>,
}

impl Record {
    /// Create a new empty record.
    pub fn new() -> Self {
        let mut v = Record::default();
        v.setup();
        v
    }

    /// Registers all types to be recorded and played back.
    fn setup(&mut self) {
        self.register_type::<components::clock::Clock>("clock");

        // To calculate the pose of actual members
        self.register_type::<components::pose::Pose>("pose");
        self.register_type::<components::pose::PreTransform>("pre_transform");
        self.register_type::<components::parent::Parent>("parent");

        // Possibly for particle emitters;
        self.register_type::<components::velocity::Velocity>("velocity");

        // self.register_type::<components::group::Group>("group");
        // self.register_type::<components::eternal::Eternal>("eternal");

        // To properly display & move tracks.
        self.register_type::<components::differential_drive_base::DifferentialDriveBase>(
            "diff_drive_base",
        );

        // Projectile visualisation.
        self.register_type::<display::tank_bullet::TankBullet>("tank_bullet");

        // Visualisation emitters that are not trivially recreated like meshes.
        self.register_type::<display::particle_emitter::ParticleEmitter>("particle_emitter");
        self.register_type::<display::deconstructor::Deconstructor>("deconstructor");

        // History of hits.
        self.register_type::<components::hit_by::HitByHistory>("hit_by_history");

        // For units, we use the unit, and the health component to track whether they should have
        // bodies.
        self.register_type::<components::health::Health>("health");
        self.register_type::<crate::units::tank::UnitTank>("unit_tank");
        self.register_type::<crate::units::artillery::UnitArtillery>("unit_artillery");
        self.register_type::<crate::units::capturable_flag::UnitCapturableFlag>(
            "unit_capturable_flag",
        );

        // Team information, to color vehicles.
        self.register_type::<components::team::Team>("team");
        self.register_type::<components::team_member::TeamMember>("team_member");

        // Capturables
        self.register_type::<components::capturable::Capturable>("capturable");
        self.register_type::<components::capture_point::CapturePoint>("capture_point");

        // Match info.
        self.register_type::<components::match_finished::MatchFinished>("match_finished");
        self.register_type::<components::match_king_of_the_hill::MatchKingOfTheHill>(
            "match_king_of_the_hill",
        );
        self.register_type::<components::match_time_limit::MatchTimeLimit>("match_time_limit");

        self.current_state.ensure_components(&self.component_map);
    }

    /// Register a particular type for recording and playback.
    fn register_type<T: Component + Serialize + DeserializeOwned + 'static>(&mut self, name: &str) {
        let component_type = self.component_map.insert(name);
        let record = Box::new(move |world_state: &mut WorldState, world: &World| {
            world_state.add_component::<T>(component_type, world);
        });
        let play = Box::new(move |delta_state: &DeltaState, world: &mut World| {
            delta_state.apply::<T>(component_type, world)
        });
        self.helpers
            .insert(component_type, TypeHandler { record, play });
    }

    /// Record the current world state, currently always writes a zipped delta state.
    pub fn record(&mut self, world: &World) {
        // Create a new empty world state.
        let mut new_world_state = WorldState::default();

        // Capture the state
        for (_, h) in self.helpers.iter() {
            (h.record)(&mut new_world_state, world);
        }

        // Determine the delta
        let delta = DeltaState::new(&self.current_state, &new_world_state);

        // Store the delta
        // self.states.push(Capture::DeltaState(delta));
        self.states
            .push(Capture::ZippedDeltaState(delta.compressed()));

        // Store previous full snap shot for next delta calculation.
        self.current_state = new_world_state;
        if let Some(v) = self.current_state_time() {
            self.max_time = Some(v);
        }
    }

    fn current_state_time(&self) -> Option<f32> {
        let clock_component = self.component_map.get("clock")?;
        // println!("clock_component: {clock_component}");
        let states = self.current_state.component_states(clock_component)?;
        let (_clock_entity, clock_data) = states.states().first()?;
        let clock = bincode::deserialize::<components::clock::Clock>(&clock_data).unwrap();
        Some(clock.elapsed_as_f32())
    }

    pub fn max_time(&self) -> Option<f32> {
        self.max_time
    }

    /// Component specific seek using the clock.
    pub fn seek(&mut self, desired: f32) {
        println!("Seeking to {desired}");
        // lets start by... just applying from the start, but no deserialization, applying dry.
        self.current_state = Default::default();
        self.current_state.ensure_components(&self.component_map);
        self.playback_index = 0;
        let mut current_time = 0.0;
        while self.playback_index < self.states.len() && current_time < desired {
            match &self.states[self.playback_index] {
                Capture::WorldState(full_state) => {
                    // println!("WorldState");
                    self.current_state = full_state.clone();
                }
                Capture::DeltaState(delta) => {
                    // println!("DeltaState");
                    delta.apply_dry(&mut self.current_state);
                }
                Capture::ZippedDeltaState(zipped_delta) => {
                    // println!("ZippedDeltaState");
                    let delta = DeltaState::uncompress(zipped_delta);
                    delta.apply_dry(&mut self.current_state);
                }
            }
            current_time = self
                .current_state_time()
                .expect("no time in current state, need at least one frame");
            self.playback_index += 1;
        }
        println!("current_time: {current_time}, {}", self.playback_index);
    }

    pub fn apply_state(&self, world: &mut World) {
        // next, create a complete delta state.
        let mut empty: WorldState = Default::default();
        empty.ensure_components(&self.component_map);
        let full_delta = DeltaState::new(&empty, &self.current_state);

        for (_, h) in self.helpers.iter() {
            (h.play)(&full_delta, world);
        }
    }

    /// Play back a step from this recording.
    pub fn step(&mut self, world: &mut World) {
        if self.playback_index < self.states.len() {
            match &self.states[self.playback_index] {
                Capture::WorldState(_full_state) => {
                    todo!()
                }
                Capture::DeltaState(delta) => {
                    for (_, h) in self.helpers.iter() {
                        (h.play)(delta, world);
                    }
                }
                Capture::ZippedDeltaState(zipped_delta) => {
                    let delta = DeltaState::uncompress(zipped_delta);
                    for (_, h) in self.helpers.iter() {
                        (h.play)(&delta, world);
                    }
                }
            }
            self.playback_index += 1;
        } else {
            // Ehh? Advance the clock manually...?
            if world
                .component_iter::<PlaybackFinishedMarker>()
                .next()
                .is_none()
            {
                // We can't create an entity, as the engine doesn't have an entity counter that's
                // in sync. Lets just add this marker to the clock entity.
                let clock_entity = world
                    .component_iter::<components::clock::Clock>()
                    .next()
                    .expect("Should have one clock")
                    .0;
                world.add_component(clock_entity, PlaybackFinishedMarker);
            }
        }
    }

    pub fn get_byte_sums(&self) -> Vec<(String, usize)> {
        let mut accumulated = std::collections::HashMap::<String, usize>::new();
        for s in self.states.iter() {
            match s {
                Capture::WorldState(full_state) => {
                    for (k, v) in self.component_map.component_map.iter() {
                        if let Some(v) = full_state.component_states(*v) {
                            *accumulated.entry(k.to_string()).or_insert(0usize) += v.sum_bytes();
                        }
                    }
                }
                Capture::DeltaState(_delta) => {
                    todo!()
                }
                Capture::ZippedDeltaState(zipped_delta) => {
                    let delta = DeltaState::uncompress(&zipped_delta);
                    for (k, v) in self.component_map.component_map.iter() {
                        if let Some(v) = delta.component_delta(*v) {
                            *accumulated.entry(k.to_string()).or_insert(0usize) += v.sum_bytes();
                        }
                    }
                }
            }
        }
        let mut r: Vec<(String, usize)> =
            accumulated.iter().map(|(k, v)| (k.clone(), *v)).collect();
        r.sort();
        r
    }
}

/// Component for the recording.
pub struct Recording {
    record: RecordStorage,
}

impl Default for Recording {
    fn default() -> Self {
        Self::new()
    }
}

impl Recording {
    pub fn new() -> Self {
        Recording {
            record: std::rc::Rc::new(std::cell::RefCell::new(Record::new())),
        }
    }

    /// Retrieve the internal storage.
    pub fn record(&self) -> RecordStorage {
        self.record.clone()
    }

    /// Load a recording from a file.
    pub fn load_file(path: &str) -> Result<Recording, Box<dyn std::error::Error>> {
        use std::io::Read;
        // let file = std::fs::File::open(path)?;

        let mut file = std::fs::File::open(path)?;
        let mut data = Vec::new();
        file.read_to_end(&mut data)?;
        Self::load_slice(&data)
    }

    /// Write a recording to a file.
    pub fn write_file(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let recording = self.record();
        let data = bincode::serialize(&*recording)?;
        use std::io::Write;
        let mut file = std::fs::File::create(path)?;
        file.write_all(&data)?;
        Ok(())
    }

    /// Load a recording from a slice.
    pub fn load_slice(data: &[u8]) -> Result<Recording, Box<dyn std::error::Error>> {
        let recorder = Self::new();
        let recording = recorder.record();
        *recording.borrow_mut() = bincode::deserialize(data)?;
        recording.borrow_mut().setup();
        Ok(recorder)
    }
}
impl Component for Recording {}
