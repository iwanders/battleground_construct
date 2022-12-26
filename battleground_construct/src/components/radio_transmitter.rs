use engine::prelude::*;

use std::collections::VecDeque;
#[derive(Debug, Clone)]
pub struct RadioTransmitter {
    config: RadioTransmitterConfig,
    next_send_time: f32,
    last_update_time: f32,
    transmission_strength: f32,
    payloads: VecDeque<Vec<u8>>,
}

#[derive(Copy, Debug, Clone)]
pub struct RadioTransmitterConfig {
    /// Interval at which the radio sends out each message.
    pub transmission_interval: f32,
    /// The transmission strength, used to calculate the receipt strength.
    pub transmission_strength_max: f32,
    /// Maximum length of each transmission, in bytes, longer messages are truncated.
    pub payload_size_limit: usize,
    /// The maximum number of pending transmissions.
    pub payload_count_limit: usize,
}

impl Default for RadioTransmitterConfig {
    fn default() -> Self {
        Self {
            transmission_strength_max: 10.0,
            transmission_interval: 0.01,
            payload_size_limit: 32,
            payload_count_limit: 16,
        }
    }
}

impl RadioTransmitter {
    pub fn new_with_config(config: RadioTransmitterConfig) -> Self {
        Self {
            config,
            next_send_time: -config.transmission_interval - 1.0,
            last_update_time: 0.0,
            transmission_strength: config.transmission_strength_max,
            payloads: VecDeque::new(),
        }
    }

    pub fn to_transmit(&mut self, current_time: f32) -> Vec<Vec<u8>> {
        let mut payloads = vec![];

        // Step through time in transmission interval steps.
        while !self.payloads.is_empty() && self.next_send_time < current_time {
            payloads.push(self.payloads.pop_front().unwrap());
            self.next_send_time = self.last_update_time + self.config.transmission_interval;
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

    pub fn transmission_strength(&self) -> f32 {
        self.transmission_strength
    }

    pub fn config(&self) -> RadioTransmitterConfig {
        self.config
    }
}
impl Component for RadioTransmitter {}

use crate::components::unit_interface::{Register, RegisterMap, UnitModule};
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
        registers.clear();
        if let Some(radio_transmitter) = world.component::<RadioTransmitter>(self.entity) {}
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_radio_transmitter() {
        let config = RadioTransmitterConfig {
            transmission_strength_max: 10.0,
            transmission_interval: 0.1,
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
