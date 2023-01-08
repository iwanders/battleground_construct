use engine::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Eternal {}

impl Default for Eternal {
    fn default() -> Self {
        Self::new()
    }
}

impl Eternal {
    pub fn new() -> Self {
        Eternal {}
    }
}
impl Component for Eternal {}
