use engine::prelude::*;

use serde::{Deserialize, Serialize};

#[derive(Copy, Deserialize, Serialize, Debug, Clone, PartialEq)]
pub enum DeployState {
    Deployed,
    InTransition,
    Normal,
}


// This must be an Rc, as we need to be able to copy it to allow a mutable world, we cannot borrow
// it out of the deploy module.
pub type DeployFunction = std::rc::Rc<dyn for<'a> Fn(&'a mut World, EntityId)>;

pub struct DeployConfig {
    pub deploy_function: DeployFunction,
}

#[derive()]
pub struct Deploy {
    config: DeployConfig,
    desired_state: DeployState,
    state: DeployState,
}

impl Deploy {
    pub fn new(config: DeployConfig) -> Self {
        Self {
            desired_state: DeployState::Normal,
            state: DeployState::Normal,
            config,
        }
    }

    pub fn set_desired_state(&mut self, new_state: DeployState) {
        self.desired_state = new_state;
    }
    pub fn get_desired_state(&self) -> DeployState {
        self.desired_state
    }

    pub fn set_state(&mut self, new_state: DeployState) {
        self.state = new_state;
    }
    pub fn get_state(&self) -> DeployState {
        self.state
    }

    pub fn deploy_function(&self) -> DeployFunction {
        self.config.deploy_function.clone()
    }
}
impl Component for Deploy {}

use crate::components::unit_interface::{Register, RegisterMap, UnitModule};

use battleground_unit_control::modules::deploy::*;

pub struct DeployModule {
    entity: EntityId,
}

impl DeployModule {
    pub fn new(entity: EntityId) -> Self {
        DeployModule { entity }
    }
}

impl UnitModule for DeployModule {
    fn get_registers(&self, world: &World, registers: &mut RegisterMap) {
        registers.clear();
        if let Some(deploy) = world.component::<Deploy>(self.entity) {
            let current = deploy.get_state();
            let finished = current == deploy.get_desired_state();
            registers.insert(
                REG_DEPLOY_FINISHED,
                Register::new_i32("finished", finished as i32),
            );
            registers.insert(
                REG_DEPLOY_DESIRED_STATE,
                Register::new_i32("desired_state", (deploy.get_desired_state() == DeployState::Deployed) as i32),
            );

            let current_as_i32 = match current {
                DeployState::Normal => DEPLOY_STATE_NORMAL,
                DeployState::Deployed => DEPLOY_STATE_DEPLOYED,
                DeployState::InTransition => DEPLOY_STATE_IN_TRANSITION,
                
            };
            registers.insert(
                REG_DEPLOY_STATE,
                Register::new_i32("state", current_as_i32),
            );
        }
    }

    fn set_component(&self, world: &mut World, registers: &RegisterMap) {
        if let Some(mut deploy) = world.component_mut::<Deploy>(self.entity) {
            let d = registers
                .get(&REG_DEPLOY_DESIRED_STATE)
                .expect("register doesnt exist")
                .value_i32()
                .expect("wrong value type");

            if d == DEPLOY_STATE_NORMAL {
                deploy.set_desired_state(DeployState::Normal);
            } else if d == DEPLOY_STATE_DEPLOYED{
                deploy.set_desired_state(DeployState::Deployed);
            }
        }
    }
}
