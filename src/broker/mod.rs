use crate::mqtt::packets::connack::ConnAckPacket;
use crate::mqtt::packets::connack::ConnAckReturnCode::Accepted;
use crate::mqtt::packets::connect::ConnectPacket;
use crate::mqtt::packets::disconnect::DisconnectPacket;
use crate::mqtt::packets::publish::PublishPacket;
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

    pub fn disconnect(&self, _disconnect_packet: &DisconnectPacket) {}

    pub fn publish(&self, _publish_packet: &PublishPacket) -> Option<()> {
        None
    }
}
