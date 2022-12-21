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

pub mod playground;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Copy, Default, Clone)]
pub struct CapturePoint {
    position: (f32, f32),
    radius: f32,
    capture_speed: f32,
    capture_acrue: f32,
    initial_team: Option<usize>,
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
    mode: MatchType,
    time_limit: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Team {
    /// Team name
    name: String,
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
    team: usize,
    vehicle: Vehicle,
    position: (f32, f32),
    orientation: f32,
    controller: ControllerType,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct SpawnConfig {
    teams: Vec<Team>,
    spawn: Vec<Spawn>,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct ConstructConfig {
    /// Denotes the match specification.
    match_config: MatchConfig,

    /// Spawn of vehicles.
    spawn_config: SpawnConfig,
}
