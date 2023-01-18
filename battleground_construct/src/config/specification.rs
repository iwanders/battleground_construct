use serde::{Deserialize, Serialize};

// Function to return a default for the capture speed.
fn default_capture_speed() -> f32 {
    1.0
}

/// Definition for a capturable point.
#[derive(Serialize, Deserialize, Debug, Copy, Default, Clone)]
pub struct CapturePoint {
    /// x coordinate of the capture point.
    pub x: f32,
    /// y coordinate of the capture point.
    pub y: f32,
    #[serde(default)]
    /// yaw of the capture point (radians).
    pub yaw: f32,
    /// Radius of the circular capture area.
    pub radius: f32,
    /// Speed at which the capture point changes owners.
    #[serde(default = "default_capture_speed")]
    pub capture_speed: f32,
    #[serde(default)]
    /// The initial owner of the capture point, index to a team.
    pub team: Option<usize>,
}

/// Specification for the match type.
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
#[serde(tag = "type")]
pub enum MatchType {
    #[default]
    /// No match type, sandbox style game without objectives.
    None,
    /// Domination game, king of the hill, but instant victory if point team deathmatch threshold
    /// achieved and all capture points are owned by any taem that has achieved the team deathmatch
    /// minimum point count.
    Domination {
        team_deathmatch_min: i64,
        point_limit: Option<f32>,
        capture_points: Vec<CapturePoint>,
    },
    /// Team Deathmatch, first team to destroy point_limit units of another team wins.
    TeamDeathmatch { point_limit: Option<i64> },
    /// King of the hill, first team to achieve point_limit points accumulated by controlling the
    /// capture points wins.
    KingOfTheHill {
        capture_points: Vec<CapturePoint>,
        point_limit: Option<f32>,
    },
}

/// Specification for a particular match
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct MatchConfig {
    /// Match type for this match.
    #[serde(default)]
    pub mode: MatchType,
    /// Optional time limit.
    pub time_limit: Option<f32>,
}

/// Specification of a team in the scenario.
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Team {
    /// Team name
    pub name: String,

    /// Team comment, like controller filename that was loaded.
    pub comment: Option<String>,

    /// Color used to represent this team. RGB; 0-255.
    pub color: (u8, u8, u8),

    /// The controller to use for this team.
    pub controller: Option<ControllerType>,
}

/// Configuration for the wasm controller.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct WasmControlConfig {
    /// Path of the file to load.
    pub path: String,

    /// Fuel (cpu allowance) per update loop.
    #[serde(default)]
    pub fuel_per_update: Option<u64>,

    /// Fuel (cpu allowance) at initial setup.
    #[serde(default)]
    pub fuel_for_setup: Option<u64>,

    /// Whether to check the path for a change in modification time and reload the module.
    #[serde(default)]
    pub reload: bool,
}
impl Default for WasmControlConfig {
    fn default() -> Self {
        Self {
            path: "".to_owned(),
            fuel_per_update: None,
            fuel_for_setup: None,
            reload: true,
        }
    }
}

/// Controller specification.
#[derive(Serialize, Deserialize, Debug, Default, Clone, PartialEq)]
#[serde(tag = "type")]
pub enum ControllerType {
    /// Idle controller does nothing.
    #[default]
    Idle,
    /// Statically rotate turret and barrel and keep shooting.
    SwivelShoot,
    /// Radio our current position with the radio transmitter.
    RadioPosition,
    /// Load a dynamic library and use that as a controller. Out of use and superseded by the wasm
    /// systems, because that allows sandboxing.
    #[cfg(not(target_arch = "wasm32"))]
    LibraryLoad { name: String },
    /// Drive forwards for a duration, then reverse direction and drive that duration backwards.
    DiffDriveForwardsBackwards {
        velocities: (f32, f32),
        duration: f32,
    },
    /// Default controller to move a differential drive base to the nearest capturable the
    /// unit doesn't own.
    DiffDriveCapturable,
    /// Builtin controller that prints the entire interface, all registers.
    InterfacePrinter,
    /// Default controller that performs very naive shooting with tank and artillery, does not
    /// account for any motion.
    NaiveShoot,
    /// Unit controller that runs a wasm module.
    #[cfg(feature = "unit_control_wasm")]
    Wasm(WasmControlConfig),
    /// Calls a function to construct a controller, can be useful when there's desire to run an
    /// external controller natively.
    #[serde(skip)]
    Function(battleground_unit_control::ControllerSpawn),
    /// A controller that executes a list of controllers in sequence.
    SequenceControl { controllers: Vec<ControllerType> },
    /// A 'redirect' to use the controller referred to by name in the controller config.
    FromControlConfig { name: String },
    /// A 'redirect' to use the controller specified in the team list.
    TeamController { name: String },
}

/// The unit type to spawn.
#[derive(Serialize, Deserialize, Debug, Copy, Default, Clone)]
pub enum Unit {
    #[default]
    Tank,
    Artillery,
}

/// Configures a unit spawn.
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Spawn {
    /// Optional team this unit is on.
    pub team: Option<usize>,
    /// The unit type to spawn here.
    #[serde(default)]
    pub unit: Unit,
    /// The x coordinate to spawn at.
    pub x: f32,
    /// The y coordinate to spawn at.
    pub y: f32,
    /// The yaw orientation to spawn with, radians.
    pub yaw: f32,
    /// Controller this unit will use, usually a redirect to either
    /// [`ControllerType::TeamController`] or [`ControllerType::FromControlConfig`].
    #[serde(default)]
    pub controller: ControllerType,

    /// The radio configuration that this unit will use, this can be used to limit communication
    /// between teams by specifying each team its own non-overlapping channel bands.
    #[serde(default)]
    pub radio: crate::units::common::RadioConfig,
}

/// Specification for spawnables.
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct SpawnConfig {
    /// Map with controller specifications.
    #[serde(default)]
    pub control_config: std::collections::HashMap<String, ControllerType>,
    /// List of teams to create.
    pub teams: Vec<Team>,
    /// List of units to spawn.
    pub spawns: Vec<Spawn>,
}

/// Specification for a scenario.
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct ScenarioConfig {
    /// String used to invoke special setup.
    #[serde(default)]
    pub pre_setup: String,

    /// When true, the recorder will record the match.
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
    /// Additional time to run update before writing recording after wrap up is called.
    pub outro: f32,

    /// Write the wrap up report to this file if a path is specified.
    pub write_wrap_up: Option<String>,

    /// Write the recording file to this path if specified.
    pub write_recording: Option<String>,

    /// The original scenario as setup.
    pub scenario: Option<ScenarioConfig>,
}
