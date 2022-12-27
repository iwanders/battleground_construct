use battleground_unit_control::modules::gps::registers as gps_registers;
use battleground_unit_control::modules::radio_transmitter::registers as radio_registers;
use battleground_unit_control::units::tank;
use battleground_unit_control::{Interface, UnitControl};

pub struct RadioPosition {}
impl UnitControl for RadioPosition {
    fn update(&mut self, interface: &mut dyn Interface) {
        let team = interface.get_i32(tank::TEAM_MODULE, 0).unwrap();

        let x = interface
            .get_f32(tank::GPS_MODULE, gps_registers::X)
            .unwrap();
        let y = interface
            .get_f32(tank::GPS_MODULE, gps_registers::Y)
            .unwrap();

        let payload_count_limit = interface
            .get_i32(
                tank::RADIO_TRANSMITTER_MODULE,
                radio_registers::PAYLOAD_COUNT_LIMIT,
            )
            .unwrap();

        let mut payload_count = interface
            .get_i32(
                tank::RADIO_TRANSMITTER_MODULE,
                radio_registers::PAYLOAD_COUNT,
            )
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
                    tank::RADIO_TRANSMITTER_MODULE,
                    radio_registers::PAYLOAD_OFFSET + payload_count as u32,
                    &payload[..],
                )
                .unwrap();
            payload_count += 1;
            interface
                .set_i32(
                    tank::RADIO_TRANSMITTER_MODULE,
                    radio_registers::PAYLOAD_COUNT,
                    payload_count,
                )
                .unwrap();
        }
    }
}
