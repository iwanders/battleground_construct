use battleground_vehicle_control::{ControllerSpawn, Interface, VehicleControl};
pub struct DynamicLoadControl {
    _lib: libloading::Library,
    controller: Option<Box<dyn VehicleControl>>,
}

impl DynamicLoadControl {
    pub fn new(lib: &str) -> Result<Box<DynamicLoadControl>, Box<dyn std::error::Error>> {
        let lib = unsafe { libloading::Library::new(lib)? };
        let res = unsafe {
            let foo: libloading::Symbol<ControllerSpawn> = lib.get(b"create_ai")?;
            foo()
        };
        Ok(Box::new(DynamicLoadControl {
            _lib: lib,
            controller: Some(res),
        }))
    }
}

impl VehicleControl for DynamicLoadControl {
    fn update(&mut self, interface: &mut dyn Interface) {
        self.controller.as_mut().unwrap().update(interface);
    }
}
