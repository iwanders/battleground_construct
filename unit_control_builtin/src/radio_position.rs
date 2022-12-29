use battleground_unit_control::modules::gps::*;
use battleground_unit_control::modules::radio_transmitter::*;
use battleground_unit_control::units::tank;
use battleground_unit_control::{Interface, UnitControl};

use crate::UnitControlResult;

pub struct RadioPosition {}
impl UnitControl for RadioPosition {
    fn update(&mut self, interface: &mut dyn Interface) -> UnitControlResult {
        let team = interface
            .get_i32(
                tank::MODULE_TANK_TEAM,
                battleground_unit_control::modules::team::REG_TEAM_TEAMID,
            )
            .unwrap();

        let x = interface.get_f32(tank::MODULE_TANK_GPS, REG_GPS_X).unwrap();
        let y = interface.get_f32(tank::MODULE_TANK_GPS, REG_GPS_Y).unwrap();

        let payload_count_limit = interface
            .get_i32(
                tank::MODULE_TANK_RADIO_TRANSMITTER,
                REG_RADIO_TX_MSG_COUNT_LIMIT,
            )
            .unwrap();

        let mut payload_count = interface
            .get_i32(tank::MODULE_TANK_RADIO_TRANSMITTER, REG_RADIO_TX_MSG_COUNT)
            .unwrap();

        // println!("payload_count: {payload_count:?}, payload_count_limit: {payload_count_limit}");
        if payload_count < payload_count_limit {
            // queue a message.
            let mut payload = [0u8; 4 * 3];
            payload[0..4].copy_from_slice(&team.to_le_bytes());
            payload[4..8].copy_from_slice(&x.to_le_bytes());
            payload[8..12].copy_from_slice(&y.to_le_bytes());
            interface
                .set_bytes(
                    tank::MODULE_TANK_RADIO_TRANSMITTER,
                    REG_RADIO_TX_MSG_START + payload_count as u32,
                    &payload[..],
                )
                .unwrap();
            payload_count += 1;
            interface
                .set_i32(
                    tank::MODULE_TANK_RADIO_TRANSMITTER,
                    REG_RADIO_TX_MSG_COUNT,
                    payload_count,
                )
                .unwrap();
        }
        Ok(())
    }
}
