use crate::components;
use crate::display;
use engine::prelude::*;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use miniz_oxide::deflate::compress_to_vec;
use miniz_oxide::inflate::decompress_to_vec;

pub struct PlaybackUnitCreatedMarker;
impl Component for PlaybackUnitCreatedMarker {}

pub struct PlaybackUnitDestroyedMarker;
impl Component for PlaybackUnitDestroyedMarker {}

pub struct PlaybackFinishedMarker;
impl Component for PlaybackFinishedMarker {}

pub type RecordingStorage = std::rc::Rc<std::cell::RefCell<Recording>>;

pub type ComponentType = usize;

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
struct ComponentMap {
    // Name to component id.
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
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
struct ComponentDelta {
    change: ComponentStates,
    removed: Vec<EntityId>,
}

impl ComponentDelta {
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
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
struct ComponentStates {
    states: Vec<(EntityId, Vec<u8>)>,
}
impl ComponentStates {
    pub fn capture<T: Component + Serialize + 'static>(world: &World) -> ComponentStates {
        let states: Vec<(EntityId, Vec<u8>)> = world
            .component_iter::<T>()
            .map(|(e, c)| (e, { bincode::serialize(&*c).unwrap() }))
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

    pub fn states(&self) -> &[(EntityId, Vec<u8>)] {
        &self.states[..]
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
struct WorldState {
    states: std::collections::HashMap<ComponentType, ComponentStates>,
}

impl WorldState {
    fn add_component<T: Component + Serialize + 'static>(
        &mut self,
        index: ComponentType,
        world: &World,
    ) {
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

    fn apply<T: Component + DeserializeOwned + 'static>(
        &self,
        component: ComponentType,
        world: &mut World,
    ) {
        if let Some(delta) = self.delta.get(&component) {
            delta.apply::<T>(world);
        }
    }

    fn compressed(&self) -> Vec<u8> {
        let bytes = bincode::serialize(self).unwrap();
        compress_to_vec(&bytes, 6)
    }
    fn uncompress(bytes: &[u8]) -> DeltaState {
        let decompressed = decompress_to_vec(bytes).expect("Failed to decompress!");
        bincode::deserialize(&decompressed).unwrap()
    }
}

#[allow(clippy::enum_variant_names)]
#[derive(Debug, Clone, Deserialize, Serialize)]
enum Capture {
    WorldState(WorldState),
    DeltaState(DeltaState),
    ZippedDeltaState(Vec<u8>),
}

type RecordFunction = Box<dyn Fn(&mut WorldState, &World)>;
type PlayFunction = Box<dyn Fn(&DeltaState, &mut World)>;

#[derive(Default, Deserialize, Serialize)]
pub struct Recording {
    component_map: ComponentMap,
    states: Vec<Capture>,
    #[serde(skip)]
    previous_state: WorldState,
    #[serde(skip)]
    playback_index: usize,
    #[serde(skip)]
    helpers: std::collections::HashMap<ComponentType, (RecordFunction, PlayFunction)>,
}
impl Recording {
    pub fn new() -> Self {
        let mut v = Recording::default();
        v.setup();
        v
    }

    fn register_type<T: Component + Serialize + DeserializeOwned + 'static>(&mut self, name: &str) {
        let component_type = self.component_map.insert(name);
        let record = Box::new(move |world_state: &mut WorldState, world: &World| {
            world_state.add_component::<T>(component_type, world);
        });
        let play = Box::new(move |delta_state: &DeltaState, world: &mut World| {
            delta_state.apply::<T>(component_type, world)
        });
        self.helpers.insert(component_type, (record, play));
    }

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

        self.previous_state.ensure_components(&self.component_map);
    }

    pub fn record(&mut self, world: &World) {
        // Create a new empty world state.
        let mut new_world_state = WorldState::default();

        // Capture the state
        for (_, (record_fun, _)) in self.helpers.iter() {
            record_fun(&mut new_world_state, world);
        }

        // Determine the delta
        let delta = DeltaState::new(&self.previous_state, &new_world_state);

        // Store the delta
        // self.states.push(Capture::DeltaState(delta));
        self.states
            .push(Capture::ZippedDeltaState(delta.compressed()));

        // Store previous full snap shot for next delta calculation.
        self.previous_state = new_world_state;
    }

    pub fn step(&mut self, world: &mut World) {
        if self.playback_index < self.states.len() {
            match &self.states[self.playback_index] {
                Capture::WorldState(_full_state) => {
                    todo!()
                }
                Capture::DeltaState(delta) => {
                    for (_, (_, play_fun)) in self.helpers.iter() {
                        play_fun(delta, world);
                    }
                }
                Capture::ZippedDeltaState(zipped_delta) => {
                    let delta = DeltaState::uncompress(zipped_delta);
                    for (_, (_, play_fun)) in self.helpers.iter() {
                        play_fun(&delta, world);
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
}

pub struct Recorder {
    recording: RecordingStorage,
}

impl Default for Recorder {
    fn default() -> Self {
        Self::new()
    }
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

    pub fn load_file(path: &str) -> Result<Recorder, Box<dyn std::error::Error>> {
        use std::io::Read;
        // let file = std::fs::File::open(path)?;

        let mut file = std::fs::File::open(path)?;
        let mut data = Vec::new();
        file.read_to_end(&mut data)?;
        Self::load_slice(&data)
    }

    pub fn load_slice(data: &[u8]) -> Result<Recorder, Box<dyn std::error::Error>> {
        let recorder = Self::new();
        let recording = recorder.recording();
        *recording.borrow_mut() = bincode::deserialize(data)?;
        recording.borrow_mut().setup();
        Ok(recorder)
    }
}
impl Component for Recorder {}
