use crate::mqtt::packets::connack::ConnAckReturnCode;
use crate::mqtt::packets::*;
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
            return_code: ConnAckReturnCode::Accepted,
        }
    }

    pub fn disconnect(&self, _disconnect_packet: &DisconnectPacket) {
        info!("Client disconnected")
    }

    pub fn publish(&self, _publish_packet: &PublishPacket) -> Option<()> {
        None
    }
}
