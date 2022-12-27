use engine::prelude::*;

#[derive(Debug, Clone)]
struct ReceivedPayload {
    strength: f32,
    payload: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct RadioReceiver {
    config: RadioReceiverConfig,
    payloads: Vec<ReceivedPayload>,
    channel: usize,
}

#[derive(Copy, Debug, Clone)]
pub struct RadioReceiverConfig {
    /// The minimum channel to be selected, transmitters on a certain channel will only be received
    /// by receivers listening on that channel.
    pub channel_min: usize,

    /// The maximum channel to be selected, transmitters on a certain channel will only be received
    /// by receivers listening on that channel.
    pub channel_max: usize,
}

impl Default for RadioReceiverConfig {
    fn default() -> Self {
        Self {
            channel_min: 0,
            channel_max: 4,
        }
    }
}

impl RadioReceiver {
    pub fn new_with_config(config: RadioReceiverConfig) -> Self {
        Self {
            config,
            payloads: vec![],
            channel: config.channel_min,
        }
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

    pub fn channel(&self) -> usize {
        self.channel
    }

    pub fn set_channel(&mut self, channel: usize) {
        self.channel = channel.clamp(self.config.channel_min, self.config.channel_max);
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
            registers.insert(
                registers::CHANNEL_MIN,
                Register::new_i32("channel_min", radio_receiver.config.channel_min as i32),
            );
            registers.insert(
                registers::CHANNEL_MAX,
                Register::new_i32("channel_max", radio_receiver.config.channel_max as i32),
            );

            registers.insert(
                registers::CHANNEL_SELECT,
                Register::new_i32("channel_select", radio_receiver.channel() as i32),
            );

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
            let channel = registers
                .get(&registers::CHANNEL_SELECT)
                .expect("register doesnt exist")
                .value_i32()
                .expect("wrong value type")
                .max(0) as usize;
            radio_receiver.set_channel(channel);

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
