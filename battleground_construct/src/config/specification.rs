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

#[derive(Serialize, Deserialize, Debug, Default, Clone, PartialEq, Eq)]
pub struct WasmControlConfig {
    pub path: String,
    #[serde(default)]
    pub fuel_per_update: Option<u64>,
    #[serde(default)]
    pub fuel_for_setup: Option<u64>,
    #[serde(default)]
    pub print_exports: bool,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone, PartialEq)]
#[serde(tag = "type")]
pub enum ControllerType {
    #[default]
    None,
    SwivelShoot,
    RadioPosition,
    #[cfg(not(target_arch = "wasm32"))]
    LibraryLoad {
        name: String,
    },
    DiffDriveForwardsBackwards {
        velocities: (f32, f32),
        duration: f32,
    },
    DiffDriveCapturable,
    InterfacePrinter,
    TankNaiveShoot,
    #[cfg(feature = "unit_control_wasm")]
    Wasm(WasmControlConfig),
    #[serde(skip)]
    Function(battleground_unit_control::ControllerSpawn),
    SequenceControl {
        controllers: Vec<ControllerType>,
    },
    FromControlConfig {
        name: String,
    },
}

#[derive(Serialize, Deserialize, Debug, Copy, Default, Clone)]
pub enum Unit {
    #[default]
    Tank,
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
    pub radio: crate::units::common::RadioConfig,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct SpawnConfig {
    #[serde(default)]
    pub control_config: std::collections::HashMap<String, ControllerType>,
    pub teams: Vec<Team>,
    pub spawns: Vec<Spawn>,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct ScenarioConfig {
    /// String used to invoke special setup.
    #[serde(default)]
    pub pre_setup: String,

    #[serde(default)]
    pub recording: bool,

    /// Denotes the match specification.
    #[serde(default)]
    pub match_config: MatchConfig,

    /// Spawn of vehicles.
    #[serde(default)]
    pub spawn_config: SpawnConfig,
}

/// This struct specifies the steps to be done after a scenario wraps up.
pub struct WrapUpConfig {
    /// Write the wrap up report to this file if a path is specified.
    pub write_wrap_up: Option<String>,

    /// Write the recording file to this path if specified.
    pub write_recording: Option<String>,

    /// The original scenario as setup.
    pub scenario: Option<ScenarioConfig>,
}
