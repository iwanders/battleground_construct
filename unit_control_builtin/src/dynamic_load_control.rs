use battleground_unit_control::{ControlError, ControllerSpawn, Interface, UnitControl};
pub struct DynamicLoadControl {
    _lib: libloading::Library,
    controller: Option<Box<dyn UnitControl>>,
}

impl DynamicLoadControl {
    pub fn new(lib: &str) -> Result<Box<DynamicLoadControl>, Box<dyn std::error::Error>> {
        let lib = unsafe { libloading::Library::new(lib)? };
        let res = unsafe {
            let create_ai: libloading::Symbol<ControllerSpawn> = lib.get(b"create_ai")?;
            create_ai()
        };
        Ok(Box::new(DynamicLoadControl {
            _lib: lib,
            controller: Some(res),
        }))
    }
}

impl UnitControl for DynamicLoadControl {
    fn update(&mut self, interface: &mut dyn Interface) -> Result<(), Box<ControlError>> {
        self.controller.as_mut().unwrap().update(interface)
    }
}
