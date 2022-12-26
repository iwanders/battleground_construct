use engine::prelude::*;

use std::collections::VecDeque;
#[derive(Debug, Clone)]
pub struct RadioTransmitter {
    config: RadioTransmitterConfig,
    next_send_time: f32,
    last_update_time: f32,
    transmit_strength: f32,
    payloads: VecDeque<Vec<u8>>,
}

#[derive(Copy, Debug, Clone)]
pub struct RadioTransmitterConfig {
    /// Maximum range for which transmissions are still delivered.
    pub transmit_range_max: f32,
    /// Interval at which the radio sends out each message.
    pub transmit_interval: f32,
    /// The transmission strength, used to calculate the receipt strength.
    pub transmit_strength_max: f32,
    /// Maximum length of each transmission, in bytes, longer messages are truncated.
    pub payload_size_limit: usize,
    /// The maximum number of pending transmissions.
    pub payload_count_limit: usize,
}

impl Default for RadioTransmitterConfig {
    fn default() -> Self {
        Self {
            transmit_range_max: 30.0,
            transmit_strength_max: 1.0,
            transmit_interval: 0.01,
            payload_size_limit: 32,
            payload_count_limit: 16,
        }
    }
}

impl RadioTransmitter {
    pub fn new_with_config(config: RadioTransmitterConfig) -> Self {
        Self {
            config,
            next_send_time: -config.transmit_interval - 1.0,
            last_update_time: 0.0,
            transmit_strength: config.transmit_strength_max,
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
        self.transmit_strength
    }

    pub fn set_transmit_strength(&mut self, strength: f32) {
        self.transmit_strength = strength.clamp(0.0, self.config.transmit_strength_max);
    }

    pub fn config(&self) -> RadioTransmitterConfig {
        self.config
    }
}
impl Component for RadioTransmitter {}

use crate::components::unit_interface::{Register, RegisterMap, UnitModule};
use battleground_unit_control::modules::radio_transmitter::registers;
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
                registers::TRANSMIT_RANGE_MAX,
                Register::new_f32(
                    "transmit_range_max",
                    radio_transmitter.config.transmit_range_max,
                ),
            );
            registers.insert(
                registers::TRANSMIT_INTERVAL,
                Register::new_f32(
                    "transmit_interval",
                    radio_transmitter.config.transmit_interval,
                ),
            );
            registers.insert(
                registers::TRANSMIT_STRENGTH_MAX,
                Register::new_f32(
                    "transmit_strength_max",
                    radio_transmitter.config.transmit_strength_max,
                ),
            );
            registers.insert(
                registers::PAYLOAD_SIZE_LIMIT,
                Register::new_i32(
                    "payload_size_limit",
                    radio_transmitter.config.payload_size_limit as i32,
                ),
            );
            registers.insert(
                registers::PAYLOAD_COUNT_LIMIT,
                Register::new_i32(
                    "payload_count_limit",
                    radio_transmitter.config.payload_count_limit as i32,
                ),
            );

            // Writeable registers.
            registers.insert(
                registers::TRANSMIT_STRENGTH,
                Register::new_f32("transmit_strength", radio_transmitter.transmit_strength),
            );

            let payloads = radio_transmitter.payloads();
            registers.insert(
                registers::PAYLOAD_COUNT,
                Register::new_i32("payload_count", payloads.len() as i32),
            );
            for i in 0..radio_transmitter.config.payload_count_limit {
                let v = registers
                    .entry(registers::PAYLOAD_OFFSET + (i as u32))
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
            let transmit_strength = registers
                .get(&registers::TRANSMIT_STRENGTH)
                .expect("register doesnt exist")
                .value_f32()
                .expect("wrong value type");
            radio_transmitter.set_transmit_strength(transmit_strength);

            let payload_count = registers
                .get(&registers::PAYLOAD_COUNT)
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
                        .get(&(registers::PAYLOAD_OFFSET + (i as u32)))
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
            transmit_strength_max: 10.0,
            transmit_interval: 0.1,
            payload_size_limit: 4,
            payload_count_limit: 3,
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
        let msgs = radio.to_transmit(t);

        // If we add three messages and advance the time by more than three intervals, we should
        // get all of them.
        t += 5.0;
        radio.set_payloads(&[&[0], &[1], &[2]]);
        let msgs = radio.to_transmit(t);
        assert_eq!(msgs.len(), 3);
        assert_eq!(radio.payloads().len(), 0);

        // Flush the radio.
        t += 10.0;
        let msgs = radio.to_transmit(t);

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
