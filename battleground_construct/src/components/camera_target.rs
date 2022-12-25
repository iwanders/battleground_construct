use engine::prelude::*;

#[derive(Copy, Debug, Clone, Default)]
pub struct CameraTarget {}

impl CameraTarget {
    pub fn new() -> Self {
        CameraTarget {}
    }
}

impl Component for CameraTarget {}
