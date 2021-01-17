use crate::mqtt::packets::connack::ConnAckPacket;
use crate::mqtt::packets::connack::ConnAckReturnCode::Accepted;
use crate::mqtt::packets::connect::ConnectPacket;
use log::info;

pub struct Manager {}

impl Manager {
    pub fn new() -> Manager {
        Manager {}
    }

    pub fn connect(&self, connect_packet: &ConnectPacket) -> ConnAckPacket {
        info!("New client {:?} connected", connect_packet.client_id);

        ConnAckPacket {
            session_present: false,
            return_code: Accepted,
        }
    }
}
