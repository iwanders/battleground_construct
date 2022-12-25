use engine::prelude::*;

#[derive(Copy, Debug, Clone, Default)]
pub struct CameraPosition {}

impl CameraPosition {
    pub fn new() -> Self {
        CameraPosition {}
    }
}

impl Component for CameraPosition {}
