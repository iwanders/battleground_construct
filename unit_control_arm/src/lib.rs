use battleground_unit_control::log;
use battleground_unit_control::{Interface, UnitControl};
use battleground_unit_control::modules::{
    clock::*, controller::*,  revolute::*, unit::*
};
// Module constants live in common and their respective units.
use battleground_unit_control::units::{common, UnitType};

// Terrible, terrible include >_<
#[path = "../../battleground_construct/src/util/cgmath.rs"]
mod cgmath;

/// Our example controller!
pub struct UnitControlExample {
    last_print: f32,
}
impl Default for UnitControlExample {
    fn default() -> Self {
        Self {
            last_print: -10000.0,
        }
    }
}

pub struct JointP{
    k_p: f32,
    set_point: f32,
    revolute: u32,
}
impl JointP {
    pub fn new(revolute: u32) -> Self{
        JointP{
            k_p: 0.5,
            set_point: 0.0,
            revolute,
        }
    }
    pub fn set_point(&mut self, v: f32) {
        self.set_point = v;
    }

    fn update(&mut self, interface: &mut dyn Interface) -> Result<(), Box<dyn std::error::Error>>  {
        let current = interface.get_f32(self.revolute, REG_REVOLUTE_POSITION)?;
        let the_error = self.set_point - current;
        let control = self.k_p * the_error;
        interface.set_f32(self.revolute, REG_REVOLUTE_VELOCITY_CMD, control)?;
        // log::info!("Setting control to : {control}", );
        Ok(())
    }
}

impl UnitControl for UnitControlExample {

    /// This function gets called periodically to control our unit.
    fn update(&mut self, interface: &mut dyn Interface) -> Result<(), Box<dyn std::error::Error>> {
        // This gets the current time.
        let t = interface.get_f32(common::MODULE_CLOCK, REG_CLOCK_ELAPSED)?;
        let PI_2 = std::f32::consts::PI / 2.0;


        let rev_base = 1;
        let rev_arm = 2;
        let rev_elbow = 3;

        let mut c_0 = JointP::new(rev_base);
        let mut c_1 = JointP::new(rev_arm);
        let mut c_2 = JointP::new(rev_elbow);
        c_1.set_point(PI_2);

        c_0.update(interface);
        c_1.update(interface);
        c_2.update(interface);
        // return Ok(());

        
        // log::info!("dskljfsldf");



        // This can be used to retrieve the unit type.
        let unit_type = interface.get_i32(common::MODULE_UNIT, REG_UNIT_UNIT_TYPE)?;
        let unit_type: UnitType = (unit_type as u32).try_into()?;

        // Every 15 seconds, dump all registers and what unit we are.
        // if (self.last_print + 15.0) < t {
            // dump_registers(interface)?;
        // }

        // Show how to draw things, that's super useful when debugging.
        draw_stuff(interface)?;

        Ok(())
    }
}

fn dump_registers(interface: &dyn Interface) -> Result<(), Box<dyn std::error::Error>> {
    use battleground_unit_control::RegisterType;
    for module_index in interface.modules()? {
        log::info!("module_name: {}", interface.module_name(module_index)?);
        let registers = interface.registers(module_index)?;
        for register_index in registers {
            let name = interface.register_name(module_index, register_index)?;
            let register_type = interface.register_type(module_index, register_index)?;
            match register_type {
                RegisterType::I32 => {
                    let v = interface.get_i32(module_index, register_index)?;
                    log::info!("    {module_index:0>8x}:{register_index:0>8x} (i32) {name}: {v}");
                }
                RegisterType::F32 => {
                    let v = interface.get_f32(module_index, register_index)?;
                    log::info!("    {module_index:0>8x}:{register_index:0>8x} (f32) {name}: {v}");
                }
                RegisterType::Bytes => {
                    let len = interface.get_bytes_len(module_index, register_index)?;
                    let mut read_v = vec![0; len];
                    interface.get_bytes(module_index, register_index, &mut read_v)?;
                    log::info!("    {module_index:0>8x}:{register_index:0>8x} (bytes {len}) {name}: {read_v:?}");
                }
            }
        }
    }
    Ok(())
}

fn draw_stuff(interface: &mut dyn Interface) -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}

#[no_mangle]
#[cfg(target_arch = "wasm32")]
pub fn create_unit_control() -> Box<dyn UnitControl> {
    Box::new(UnitControlExample::default())
}
