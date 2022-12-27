use engine::prelude::*;

use std::collections::VecDeque;
#[derive(Debug, Clone)]
pub struct RadioTransmitter {
    config: RadioTransmitterConfig,
    next_send_time: f32,
    last_update_time: f32,
    payloads: VecDeque<Vec<u8>>,
    channel: usize,
}

#[derive(Copy, Debug, Clone)]
pub struct RadioTransmitterConfig {
    /// Maximum range for which transmissions are still delivered.
    pub transmit_range_max: f32,

    /// Interval at which the radio sends out each message.
    pub transmit_interval: f32,

    /// Maximum length of each transmission, in bytes, longer messages are truncated.
    pub payload_size_limit: usize,

    /// The maximum number of pending transmissions.
    pub payload_count_limit: usize,

    /// The minimum channel to be selected, transmitters on a certain channel will only be received
    /// by receivers listening on that channel.
    pub channel_min: usize,

    /// The maximum channel to be selected, transmitters on a certain channel will only be received
    /// by receivers listening on that channel.
    pub channel_max: usize,
}

impl Default for RadioTransmitterConfig {
    fn default() -> Self {
        Self {
            transmit_range_max: 30.0,
            transmit_interval: 0.01,
            payload_size_limit: 32,
            payload_count_limit: 16,
            channel_min: 0,
            channel_max: 4,
        }
    }
}

impl RadioTransmitter {
    pub fn new_with_config(config: RadioTransmitterConfig) -> Self {
        Self {
            config,
            next_send_time: -config.transmit_interval - 1.0,
            last_update_time: 0.0,
            channel: config.channel_min,
            payloads: VecDeque::new(),
        }
    }

    pub fn to_transmit(&mut self, current_time: f32) -> Vec<Vec<u8>> {
        let mut payloads = vec![];

        // Step through time in transmission interval steps.
        while !self.payloads.is_empty() && self.next_send_time < current_time {
            payloads.push(self.payloads.pop_front().unwrap());
            self.next_send_time = self.last_update_time + self.config.transmit_interval;
        }
        self.last_update_time = current_time;
        payloads
    }

    /// Adding payloads counts as if they were added at the last update time.
    pub fn set_payloads(&mut self, payloads: &[&[u8]]) {
        self.payloads = payloads
            .iter()
            .take(self.config.payload_count_limit)
            .map(|v| v[0..v.len().min(self.config.payload_size_limit)].to_vec())
            .collect();
    }

    pub fn payloads(&self) -> Vec<Vec<u8>> {
        self.payloads.clone().into()
    }

    pub fn transmit_strength(&self) -> f32 {
        1.0
    }

    pub fn channel(&self) -> usize {
        self.channel
    }

    pub fn set_channel(&mut self, channel: usize) {
        self.channel = channel.clamp(self.config.channel_min, self.config.channel_max);
    }

    pub fn config(&self) -> RadioTransmitterConfig {
        self.config
    }
}
impl Component for RadioTransmitter {}

use crate::components::unit_interface::{Register, RegisterMap, UnitModule};
use battleground_unit_control::modules::radio_transmitter::*;
pub struct RadioTransmitterModule {
    entity: EntityId,
}

impl RadioTransmitterModule {
    pub fn new(entity: EntityId) -> Self {
        RadioTransmitterModule { entity }
    }
}

impl UnitModule for RadioTransmitterModule {
    fn get_registers(&self, world: &World, registers: &mut RegisterMap) {
        // No clear here, we always overwrite all registers.
        if let Some(radio_transmitter) = world.component::<RadioTransmitter>(self.entity) {
            // Read only
            registers.insert(
                REG_RADIO_TX_RANGE_MAX,
                Register::new_f32(
                    "transmit_range_max",
                    radio_transmitter.config.transmit_range_max,
                ),
            );
            registers.insert(
                REG_RADIO_TX_INTERVAL,
                Register::new_f32(
                    "transmit_interval",
                    radio_transmitter.config.transmit_interval,
                ),
            );
            registers.insert(
                REG_RADIO_TX_MSG_SIZE_LIMIT,
                Register::new_i32(
                    "payload_size_limit",
                    radio_transmitter.config.payload_size_limit as i32,
                ),
            );
            registers.insert(
                REG_RADIO_TX_MSG_COUNT_LIMIT,
                Register::new_i32(
                    "payload_count_limit",
                    radio_transmitter.config.payload_count_limit as i32,
                ),
            );
            registers.insert(
                REG_RADIO_TX_CHANNEL_MIN,
                Register::new_i32("channel_min", radio_transmitter.config.channel_min as i32),
            );
            registers.insert(
                REG_RADIO_TX_CHANNEL_MAX,
                Register::new_i32("channel_max", radio_transmitter.config.channel_max as i32),
            );

            // Writeable registers.
            registers.insert(
                REG_RADIO_TX_CHANNEL_SELECT,
                Register::new_i32("channel_select", radio_transmitter.channel() as i32),
            );

            let payloads = radio_transmitter.payloads();
            registers.insert(
                REG_RADIO_TX_MSG_COUNT,
                Register::new_i32("payload_count", payloads.len() as i32),
            );
            for i in 0..radio_transmitter.config.payload_count_limit {
                let v = registers
                    .entry(REG_RADIO_TX_MSG_START + (i as u32))
                    .or_insert(Register::new_bytes_max(
                        "payload",
                        radio_transmitter.config.payload_size_limit,
                    ));
                let value = if i < payloads.len() {
                    payloads[i].clone()
                } else {
                    vec![]
                };
                *v.value_bytes_mut().unwrap() = value;
            }
        }
    }

    fn set_component(&self, world: &mut World, registers: &RegisterMap) {
        if let Some(mut radio_transmitter) = world.component_mut::<RadioTransmitter>(self.entity) {
            let channel = registers
                .get(&REG_RADIO_TX_CHANNEL_SELECT)
                .expect("register doesnt exist")
                .value_i32()
                .expect("wrong value type")
                .max(0) as usize;
            radio_transmitter.set_channel(channel);

            let payload_count = registers
                .get(&REG_RADIO_TX_MSG_COUNT)
                .expect("register doesnt exist")
                .value_i32()
                .expect("wrong value type")
                .max(0) as usize;

            // Finally, collect the payloads
            let upper = radio_transmitter
                .config
                .payload_count_limit
                .min(payload_count);
            let payloads = (0..upper)
                .map(|i| {
                    registers
                        .get(&(REG_RADIO_TX_MSG_START + (i as u32)))
                        .unwrap()
                        .value_bytes()
                        .unwrap()
                })
                .collect::<Vec<&[u8]>>();
            radio_transmitter.set_payloads(&payloads);
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_radio_transmitter() {
        let config = RadioTransmitterConfig {
            transmit_range_max: 1.0,
            transmit_interval: 0.1,
            payload_size_limit: 4,
            payload_count_limit: 3,
            channel_min: 0,
            channel_max: 0,
        };
        let mut radio = RadioTransmitter::new_with_config(config);
        radio.set_payloads(&[&[0], &[1], &[2]]);
        let mut t = 0.0;

        let msgs = radio.to_transmit(t);
        assert_eq!(msgs.len(), 1);
        assert_eq!(radio.payloads().len(), 2);

        // Calling transmit again without any time change, means nothing should happen.
        let msgs = radio.to_transmit(t);
        assert_eq!(msgs.len(), 0);
        assert_eq!(radio.payloads().len(), 2);
        t += 0.05;
        // Half an interval later, still nothing should change.
        let msgs = radio.to_transmit(t);
        assert_eq!(msgs.len(), 0);
        assert_eq!(radio.payloads().len(), 2);

        // Full interval later, we should get one message.
        t += 0.1;
        let msgs = radio.to_transmit(t);
        assert_eq!(msgs.len(), 1);
        assert_eq!(radio.payloads().len(), 1);

        // Now, little over half an interval later, we should get the last message.
        t += 0.06;
        let msgs = radio.to_transmit(t);
        assert_eq!(msgs.len(), 1);
        assert_eq!(radio.payloads().len(), 0);

        // Now, if we idle for like 10 seconds, our last send is way in the past.
        t += 10.0;
        let msgs = radio.to_transmit(t);
        assert_eq!(msgs.len(), 0);
        assert_eq!(radio.payloads().len(), 0);

        // Now, adding payloads should only allow a single to be sent.
        radio.set_payloads(&[&[0], &[1], &[2]]);
        t += 0.1;
        let msgs = radio.to_transmit(t);
        assert_eq!(msgs.len(), 1);
        assert_eq!(radio.payloads().len(), 2);
        // Advancing more than two intervals should allow all to be sent.
        t += 0.5;
        let msgs = radio.to_transmit(t);
        assert_eq!(msgs.len(), 2);
        assert_eq!(radio.payloads().len(), 0);

        // Radio is now empty, advance time,
        t += 10.0;
        let _msgs = radio.to_transmit(t);

        // If we add three messages and advance the time by more than three intervals, we should
        // get all of them.
        t += 5.0;
        radio.set_payloads(&[&[0], &[1], &[2]]);
        let msgs = radio.to_transmit(t);
        assert_eq!(msgs.len(), 3);
        assert_eq!(radio.payloads().len(), 0);

        // Flush the radio.
        t += 10.0;
        let _msgs = radio.to_transmit(t);

        // Add payloads, first message is too long, and there's too many payloads.
        radio.set_payloads(&[&[1, 2, 3, 4, 5, 6], &[1], &[2], &[3], &[4]]);
        t += 0.1;
        let msgs = radio.to_transmit(t);
        assert_eq!(msgs.len(), 1);
        assert_eq!(msgs[0].len(), 4);
        assert_eq!(msgs[0], [1, 2, 3, 4]);
        assert_eq!(radio.payloads().len(), 2); // 3 is the limit, 1 got retrieved.
    }
}
