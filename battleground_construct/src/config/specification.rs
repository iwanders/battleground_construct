/*
struct ClockConfig {
    pub step: f32,
}

#[derive(Default)]
struct ComponentConfig {
    clock: Option<ClockConfig>,
    radar: Option<crate::components::radar::Radar>,
    controller: Option<ControllerConfig>
    // ...
}
// Lets not over-engineer this from the get-go.
*/
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Copy, Default, Clone)]
pub struct CapturePoint {
    pub position: (f32, f32),
    pub radius: f32,
    pub capture_speed: f32,
    pub capture_acrue: f32,
    pub initial_team: Option<usize>,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub enum MatchType {
    #[default]
    None,
    DeathMatch,
    KingOfTheHill {
        capture_points: Vec<CapturePoint>,
        point_limit: Option<f32>,
    },
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct MatchConfig {
    #[serde(default)]
    pub mode: MatchType,
    pub time_limit: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Team {
    /// Team name
    pub name: String,
    // color: (f32, f32, f32),
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub enum ControllerType {
    #[default]
    None,
    LibraryLoad {
        name: String,
    },
    ForwardBackward {
        velocities: (f32, f32),
        duration: f32,
    },
    #[serde(skip)]
    Function(battleground_vehicle_control::ControllerSpawn),
}

#[derive(Serialize, Deserialize, Debug, Copy, Default, Clone)]
pub enum Vehicle {
    #[default]
    Tank,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Spawn {
    pub team: Option<usize>,
    #[serde(default)]
    pub vehicle: Vehicle,
    pub x: f32,
    pub y: f32,
    pub yaw: f32,
    #[serde(default)]
    pub controller: ControllerType,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct SpawnConfig {
    pub teams: Vec<Team>,
    pub spawns: Vec<Spawn>,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct ConstructConfig {
    /// Denotes the match specification.
    #[serde(default)]
    pub match_config: MatchConfig,

    /// Spawn of vehicles.
    pub spawn_config: SpawnConfig,
}
