use super::components::differential_drive_base::DifferentialDriveBase;
use super::display::draw_kinematic_chain_diff_drive::DrawKinematicChainDiffDrive;
use engine::prelude::*;


pub struct DrawKinematicChain {}
impl System for DrawKinematicChain {
    fn update(&mut self, world: &mut World) {
        let mut add_draw_diff_drive = vec![];
        for (entity, base) in world.component_iter::<DifferentialDriveBase>() {
            if let Some(mut draw_diff_drive) = world.component_mut::<DrawKinematicChainDiffDrive>(entity) {
                draw_diff_drive.update(&base);
            } else {
                add_draw_diff_drive.push(entity)
            }
        }

        for v in add_draw_diff_drive {
            world.add_component(v, DrawKinematicChainDiffDrive::default());
        }
    }
}
