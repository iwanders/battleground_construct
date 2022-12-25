use engine::prelude::*;

#[derive(Copy, Debug, Clone, Default)]
pub struct CaptureMarker {}

impl CaptureMarker {
    pub fn new() -> Self {
        CaptureMarker {}
    }
}

impl Component for CaptureMarker {}
