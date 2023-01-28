use super::components::differential_drive_base::DifferentialDriveBase;
use super::components::pose::{Pose, PreTransform};
use super::components::revolute::Revolute;
use super::display::draw_kinematic_chain_diff_drive::DrawKinematicChainDiffDrive;
use super::display::draw_kinematic_chain_revolute::DrawKinematicChainRevolute;
use engine::prelude::*;

pub struct DrawKinematicChain {}
impl System for DrawKinematicChain {
    fn update(&mut self, world: &mut World) {
        let mut add_diff_drive = vec![];
        for (entity, base) in world.component_iter::<DifferentialDriveBase>() {
            if let Some(mut draw_diff_drive) =
                world.component_mut::<DrawKinematicChainDiffDrive>(entity)
            {
                draw_diff_drive.update(&base);
            } else {
                add_diff_drive.push(entity);
            }
        }

        let mut add_revolute = vec![];
        for (entity, base) in world.component_iter::<Revolute>() {
            if let Some(mut draw_revolute) =
                world.component_mut::<DrawKinematicChainRevolute>(entity)
            {
                let pre = world.component::<PreTransform>(entity);
                let pre = pre.as_deref();
                let pose = world.component::<Pose>(entity);
                let pose = pose.as_deref();
                draw_revolute.update(pre, pose, &base);
            } else {
                add_revolute.push(entity);
            }
        }

        for v in add_diff_drive {
            world.add_component(v, DrawKinematicChainDiffDrive::default());
        }
        for v in add_revolute {
            world.add_component(v, DrawKinematicChainRevolute::default());
        }
    }
}
