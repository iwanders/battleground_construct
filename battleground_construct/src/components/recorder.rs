use engine::prelude::*;

pub type RecordingStorage = std::rc::Rc<std::cell::RefCell<Recording>>;

#[derive(Debug, Clone)]
pub struct Recording {}
impl Recording {
    pub fn new() -> Self {
        Recording {}
    }

    pub fn record(&mut self, world: &World) {
        // println!("yay");
    }
}

#[derive(Debug, Clone)]
pub struct Recorder {
    recording: RecordingStorage,
}

impl Recorder {
    pub fn new() -> Self {
        Recorder {
            recording: std::rc::Rc::new(std::cell::RefCell::new(Recording::new())),
        }
    }

    pub fn recording(&self) -> RecordingStorage {
        self.recording.clone()
    }
}
impl Component for Recorder {}
