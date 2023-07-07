use super::components;
use super::components::differential_drive_base::DifferentialDriveBase;
use super::components::pose::{Pose, PreTransform};
use super::components::revolute::Revolute;
use super::components::tricycle_base::TricycleBase;
use super::display::draw_kinematic_chain_diff_drive::DrawKinematicChainDiffDrive;
use super::display::draw_kinematic_chain_effector::DrawKinematicChainEffector;
use super::display::draw_kinematic_chain_link::DrawKinematicChainLink;
use super::display::draw_kinematic_chain_tricycle::DrawKinematicChainTricycle;
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

        let mut add_tricycle_drive = vec![];
        for (entity, base) in world.component_iter::<TricycleBase>() {
            if let Some(mut draw_tricycle_drive) =
                world.component_mut::<DrawKinematicChainTricycle>(entity)
            {
                draw_tricycle_drive.update(&base);
            } else {
                add_tricycle_drive.push(entity);
            }
        }

        let mut add_chain_link = vec![];

        for (entity, _parent) in world.component_iter::<components::parent::Parent>() {
            if let Some(mut draw_chain_link) = world.component_mut::<DrawKinematicChainLink>(entity)
            {
                let pose = if let Some(p) = world.component::<Pose>(entity) {
                    *p
                } else {
                    Pose::default()
                };
                let pre = world.component::<PreTransform>(entity);
                let pre = pre.as_deref();

                let revolute = world.component::<Revolute>(entity);
                let revolute = revolute.as_deref();

                let radar = world.component::<components::radar::Radar>(entity);
                let radar = radar.as_deref();
                let cannon = world.component::<components::cannon::Cannon>(entity);
                let cannon = cannon.as_deref();
                let gun_battery = world.component::<components::gun_battery::GunBattery>(entity);
                let gun_battery = gun_battery.as_deref();
                if radar.is_some()
                    || cannon.is_some()
                    || gun_battery.is_some()
                    || revolute.is_some()
                {
                    draw_chain_link.update(&pose, pre, revolute);
                }
            } else {
                add_chain_link.push(entity);
            }
        }

        let mut add_effector = vec![];
        for (entity, cannon) in world.component_iter::<components::cannon::Cannon>() {
            if let Some(mut effector) = world.component_mut::<DrawKinematicChainEffector>(entity) {
                effector.update_cannon(&cannon);
            } else {
                add_effector.push(entity);
            }
        }

        for (entity, radar) in world.component_iter::<components::radar::Radar>() {
            if let Some(mut effector) = world.component_mut::<DrawKinematicChainEffector>(entity) {
                effector.update_radar(&radar);
            } else {
                add_effector.push(entity);
            }
        }

        for (entity, gun_battery) in world.component_iter::<components::gun_battery::GunBattery>() {
            if let Some(mut effector) = world.component_mut::<DrawKinematicChainEffector>(entity) {
                effector.update_gun_battery(&gun_battery);
            } else {
                add_effector.push(entity);
            }
        }

        for v in add_diff_drive {
            world.add_component(v, DrawKinematicChainDiffDrive::default());
        }
        for v in add_tricycle_drive {
            world.add_component(v, DrawKinematicChainTricycle::default());
        }
        for v in add_chain_link {
            world.add_component(v, DrawKinematicChainLink::default());
        }
        for v in add_effector {
            world.add_component(v, DrawKinematicChainEffector::default());
        }
    }
}
