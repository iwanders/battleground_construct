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

fn default_capture_speed() -> f32 {
    1.0
}
#[derive(Serialize, Deserialize, Debug, Copy, Default, Clone)]
pub struct CapturePoint {
    pub x: f32,
    pub y: f32,
    #[serde(default)]
    pub yaw: f32,
    pub radius: f32,
    #[serde(default = "default_capture_speed")]
    pub capture_speed: f32,
    #[serde(default)]
    pub team: Option<usize>,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
#[serde(tag = "type")]
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
    /// Color used to represent this team. RGB; 0-255.
    pub color: (u8, u8, u8),
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
#[serde(tag = "type")]
pub enum ControllerType {
    #[default]
    None,
    SwivelShoot,
    LibraryLoad {
        name: String,
    },
    ForwardBackward {
        velocities: (f32, f32),
        duration: f32,
    },
    DiffDriveCapturable,
    #[cfg(feature = "unit_control_wasm")]
    Wasm {
        module: String,
    },
    #[serde(skip)]
    Function(battleground_unit_control::ControllerSpawn),
}

#[derive(Serialize, Deserialize, Debug, Copy, Default, Clone)]
pub enum Unit {
    #[default]
    Tank,
}

/// Radio config, for both transmitter and receiver.
#[derive(Serialize, Deserialize, Debug, Default, Clone, Copy)]
pub struct RadioConfig {
    pub channel_min: usize,
    pub channel_max: usize,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Spawn {
    pub team: Option<usize>,
    #[serde(default)]
    pub vehicle: Unit,
    pub x: f32,
    pub y: f32,
    pub yaw: f32,
    #[serde(default)]
    pub controller: ControllerType,
    #[serde(default)]
    pub radio: RadioConfig,
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
