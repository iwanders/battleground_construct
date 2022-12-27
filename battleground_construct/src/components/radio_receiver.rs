use engine::prelude::*;

#[derive(Debug, Clone)]
struct ReceivedMessage {
    strength: f32,
    message: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct RadioReceiver {
    config: RadioReceiverConfig,
    messages: Vec<ReceivedMessage>,
    channel: usize,
    message_overflow: usize,
}

#[derive(Copy, Debug, Clone)]
pub struct RadioReceiverConfig {
    /// The minimum channel to be selected, transmitters on a certain channel will only be received
    /// by receivers listening on that channel.
    pub channel_min: usize,

    /// The maximum channel to be selected, transmitters on a certain channel will only be received
    /// by receivers listening on that channel.
    pub channel_max: usize,

    /// The maximum number of incoming transmissions.
    pub message_count_limit: usize,
}

impl Default for RadioReceiverConfig {
    fn default() -> Self {
        Self {
            channel_min: 0,
            channel_max: 4,
            message_count_limit: 64,
        }
    }
}

impl RadioReceiver {
    pub fn new_with_config(config: RadioReceiverConfig) -> Self {
        Self {
            config,
            messages: vec![],
            channel: config.channel_min,
            message_overflow: 0,
        }
    }

    pub fn add_message(&mut self, strength: f32, message: &[u8]) {
        if self.messages.len() < self.config.message_count_limit {
            self.messages.push(ReceivedMessage {
                strength,
                message: message.to_vec(),
            })
        } else {
            self.message_overflow += 1;
        }
    }

    pub fn clear(&mut self) {
        self.messages.clear()
    }

    fn messages(&self) -> &[ReceivedMessage] {
        &self.messages[..]
    }

    pub fn channel(&self) -> usize {
        self.channel
    }

    pub fn set_channel(&mut self, channel: usize) {
        self.channel = channel.clamp(self.config.channel_min, self.config.channel_max);
    }
    pub fn set_message_overflow(&mut self, message_overflow: usize) {
        self.message_overflow = message_overflow;
    }
}
impl Component for RadioReceiver {}

use crate::components::unit_interface::{Register, RegisterMap, UnitModule};
use battleground_unit_control::modules::radio_receiver::*;
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
                REG_RADIO_RX_CHANNEL_MIN,
                Register::new_i32("channel_min", radio_receiver.config.channel_min as i32),
            );
            registers.insert(
                REG_RADIO_RX_CHANNEL_MAX,
                Register::new_i32("channel_max", radio_receiver.config.channel_max as i32),
            );

            registers.insert(
                REG_RADIO_RX_MSG_COUNT_LIMIT,
                Register::new_i32(
                    "message_count_limit",
                    radio_receiver.config.message_count_limit as i32,
                ),
            );

            registers.insert(
                REG_RADIO_RX_CHANNEL_SELECT,
                Register::new_i32("channel_select", radio_receiver.channel() as i32),
            );

            registers.insert(
                REG_RADIO_RX_MSG_OVERFLOW,
                Register::new_i32("message_overflow", radio_receiver.message_overflow as i32),
            );

            let messages = radio_receiver.messages();
            registers.insert(
                REG_RADIO_RX_MSG_COUNT,
                Register::new_i32("message_count", messages.len() as i32),
            );

            for i in 0..messages.len() {
                registers.insert(
                    REG_RADIO_RX_MSG_START
                        + (REG_RADIO_RX_MSG_STRIDE * i as u32)
                        + REG_RADIO_RX_MSG_OFFSET_STRENGTH,
                    Register::new_f32("message_strength", messages[i].strength),
                );
                let v = registers
                    .entry(
                        REG_RADIO_RX_MSG_START
                            + (REG_RADIO_RX_MSG_STRIDE * i as u32)
                            + REG_RADIO_RX_MSG_OFFSET_DATA,
                    )
                    .or_insert(Register::new_bytes("message_data"));
                *v.value_bytes_mut().unwrap() = messages[i].message.clone();
            }
        }
    }

    fn set_component(&self, world: &mut World, registers: &RegisterMap) {
        if let Some(mut radio_receiver) = world.component_mut::<RadioReceiver>(self.entity) {
            let channel = registers
                .get(&REG_RADIO_RX_CHANNEL_SELECT)
                .expect("register doesnt exist")
                .value_i32()
                .expect("wrong value type")
                .max(0) as usize;
            radio_receiver.set_channel(channel);

            let new_message_overflow = registers
                .get(&REG_RADIO_RX_MSG_OVERFLOW)
                .expect("register doesnt exist")
                .value_i32()
                .expect("wrong value type")
                .max(0) as usize;
            radio_receiver.set_message_overflow(new_message_overflow);

            let payload_count = registers
                .get(&REG_RADIO_RX_MSG_COUNT)
                .expect("register doesnt exist")
                .value_i32()
                .expect("wrong value type");
            if payload_count == 0 {
                radio_receiver.clear();
            }
        }
    }
}
