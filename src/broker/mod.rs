use crate::mqtt::packets::ConnAckReturnCode::Accepted;
use crate::mqtt::packets::{ConnAckPacket, ConnectPacket};

pub struct Manager {}

impl Manager {
    pub fn new() -> Manager {
        Manager {}
    }

    pub fn connect(&self, connect_packet: &ConnectPacket) -> ConnAckPacket {
        println!(
            "New client '{}' connected",
            connect_packet.client_identifier
        );

        ConnAckPacket {
            session_present: false,
            return_code: Accepted,
        }
    }
}
