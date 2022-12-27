use super::components::clock::Clock;
use super::components::pose::world_pose;
use super::components::radio_receiver::RadioReceiver;
use super::components::radio_transmitter::RadioTransmitter;
use crate::util::cgmath::EuclideanNorm;
use crate::util::cgmath::ToTranslation;
use engine::prelude::*;

pub struct RadioTransmission {}
impl System for RadioTransmission {
    fn update(&mut self, world: &mut World) {
        let t = world
            .component_iter_mut::<Clock>()
            .next()
            .expect("Should have one clock")
            .1
            .elapsed_as_f32();

        struct Transmission {
            entity: EntityId,
            msgs: Vec<Vec<u8>>,
            pos: cgmath::Vector3<f32>,
            strength: f32,
            transmit_max_range: f32,
        }

        let mut pending_transmissions: Vec<Transmission> = vec![];
        for (entity, mut transmitter) in world.component_iter_mut::<RadioTransmitter>() {
            let msgs = transmitter.to_transmit(t);
            if !msgs.is_empty() {
                let pose = world_pose(world, entity);
                pending_transmissions.push(Transmission {
                    entity,
                    msgs,
                    pos: pose.to_translation(),
                    strength: transmitter.transmit_strength(),
                    transmit_max_range: transmitter.config().transmit_range_max,
                });
            }
        }

        for (entity, mut receiver) in world.component_iter_mut::<RadioReceiver>() {
            let receiver_pose = world_pose(world, entity).to_translation();
            for transmission in pending_transmissions.iter() {
                if transmission.entity == entity {
                    continue; // a receiver attached to this transmitter, lets not deliver echoes.
                }
                let distance = (transmission.pos - receiver_pose).euclid_norm();
                if distance < transmission.transmit_max_range {
                    // calculate the strength.
                    let ratio_towards = 1.0 / distance.powi(2);
                    let total_strength = transmission.strength * ratio_towards;
                    for payload in transmission.msgs.iter() {
                        receiver.add_payload(total_strength, &payload[..]);
                    }
                }
            }
        }
    }
}
