use serde::{Deserialize, Serialize};

/// Radio config, for both transmitter and receiver.
#[derive(Serialize, Deserialize, Debug, Default, Clone, Copy)]
pub struct RadioConfig {
    pub channel_min: usize,
    pub channel_max: usize,
}
