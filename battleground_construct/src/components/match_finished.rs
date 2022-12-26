use engine::prelude::*;

#[derive(Copy, Debug, Clone)]
pub struct MatchFinished {}

impl MatchFinished {
    pub fn new() -> Self {
        Self {}
    }
}
impl Component for MatchFinished {}
