use engine::prelude::*;

#[derive(Debug, Clone)]
struct ReceivedPayload {
    strength: f32,
    payload: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct RadioReceiver {
    payloads: Vec<ReceivedPayload>,
}

/*
#[derive(Copy, Debug, Clone)]
pub struct RadioReceiverConfig {
    /// Minimum strength at which messages can still be received.
    pub receive_floor: f32,
}

impl Default for RadioTransmitterConfig {
    fn default() -> Self {
        Self {
            receive_floor: 0.0
        }
    }
}
*/

impl RadioReceiver {
    pub fn new() -> Self {
        Self { payloads: vec![] }
    }

    pub fn add_payload(&mut self, strength: f32, payload: &[u8]) {
        self.payloads.push(ReceivedPayload {
            strength,
            payload: payload.to_vec(),
        })
    }

    pub fn clear(&mut self) {
        self.payloads.clear()
    }

    fn payloads(&self) -> &[ReceivedPayload] {
        &self.payloads[..]
    }
}
impl Component for RadioReceiver {}

use crate::components::unit_interface::{Register, RegisterMap, UnitModule};
use battleground_unit_control::modules::radio_receiver::registers;
pub struct RadioReceiverModule {
    entity: EntityId,
}

impl RadioReceiverModule {
    pub fn new(entity: EntityId) -> Self {
        RadioReceiverModule { entity }
    }
}

impl UnitModule for RadioReceiverModule {
    fn get_registers(&self, world: &World, registers: &mut RegisterMap) {
        registers.clear();
        if let Some(radio_receiver) = world.component::<RadioReceiver>(self.entity) {
            let payloads = radio_receiver.payloads();
            registers.insert(
                registers::PAYLOAD_COUNT,
                Register::new_i32("payload_count", radio_receiver.payloads().len() as i32),
            );
            for i in 0..payloads.len() {
                registers.insert(
                    registers::PAYLOAD_START
                        + (registers::PAYLOAD_STRIDE * i as u32)
                        + registers::PAYLOAD_OFFSET_STRENGTH,
                    Register::new_f32("payload_strength", payloads[i].strength),
                );
                let v = registers
                    .entry(
                        registers::PAYLOAD_START
                            + (registers::PAYLOAD_STRIDE * i as u32)
                            + registers::PAYLOAD_OFFSET_DATA,
                    )
                    .or_insert(Register::new_bytes("payload_data"));
                *v.value_bytes_mut().unwrap() = payloads[i].payload.clone();
            }
        }
    }

    fn set_component(&self, world: &mut World, registers: &RegisterMap) {
        if let Some(mut radio_receiver) = world.component_mut::<RadioReceiver>(self.entity) {
            let payload_count = registers
                .get(&registers::PAYLOAD_COUNT)
                .expect("register doesnt exist")
                .value_i32()
                .expect("wrong value type");
            if payload_count == 0 {
                radio_receiver.clear();
            }
        }
    }
}
