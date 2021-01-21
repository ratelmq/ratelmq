use crate::mqtt::packets::connack::ConnAckPacket;
use crate::mqtt::packets::connect::ConnectPacket;
use crate::mqtt::packets::disconnect::DisconnectPacket;
use crate::mqtt::packets::publish::PublishPacket;
use crate::mqtt::packets::ControlPacket::{ConnAck, Connect, Disconnect, Publish};
use crate::mqtt::packets::ProtocolVersion::Mqtt3;
use crate::mqtt::packets::QoS::AtMostOnce;

pub mod connack;
pub mod connect;
pub mod disconnect;
pub mod publish;

#[derive(Debug, PartialEq, Clone)]
pub enum ProtocolVersion {
    Mqtt3,
    Mqtt5,
}

impl Default for ProtocolVersion {
    fn default() -> Self {
        Mqtt3
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum QoS {
    AtMostOnce = 0,
    AtLeastOnce = 1,
    ExactlyOnce = 2,
}

impl Default for QoS {
    fn default() -> Self {
        AtMostOnce
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum ControlPacket {
    Connect(ConnectPacket),
    ConnAck(ConnAckPacket),
    Publish(PublishPacket),
    Disconnect(DisconnectPacket),
}

impl ControlPacket {
    pub fn new(packet_id: u8) -> ControlPacket {
        match packet_id {
            1 => Connect(ConnectPacket::default()),
            2 => ConnAck(ConnAckPacket::default()),
            3 => Publish(PublishPacket::default()),
            14 => Disconnect(DisconnectPacket::default()),
            _ => panic!("Invalid packet id!"),
        }
    }
}
