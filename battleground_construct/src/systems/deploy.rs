use super::components::deploy::Deploy as DeployComponent;
use engine::prelude::*;

pub struct Deploy {}
impl System for Deploy {
    fn update(&mut self, world: &mut World) {
        for deploy_entity in world.component_entities::<DeployComponent>() {
            let deploy_effect = {
                world
                    .component::<DeployComponent>(deploy_entity)
                    .unwrap()
                    .deploy_function()
            };
            deploy_effect(world, deploy_entity);
        }
    }
}
