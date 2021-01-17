use num_enum::FromPrimitive;

use crate::mqtt::packets::connack::ConnAckPacket;
use crate::mqtt::packets::connect::ConnectPacket;
use crate::mqtt::packets::ControlPacket::{ConnAck, Connect};
use crate::mqtt::packets::ProtocolVersion::Mqtt3;
use crate::mqtt::packets::QoS::AtMostOnce;

pub const FIXED_HEADER_MAX_SIZE: usize = 1 + 4;

pub mod connack;
pub mod connect;

#[derive(Eq, PartialEq, Debug, FromPrimitive)]
#[repr(u8)]
pub enum ControlPacketType {
    #[num_enum(default)]
    Reserved0,
    Connect,
    ConnAck,
    Publish,
    PubAck,
    PubRec,
    PubRel,
    PubComp,
    Subscribe,
    SubAck,
    Unsubscribe,
    UnsubAck,
    PingReq,
    PingResp,
    Disconnect,
    Reserved15,
}

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
    Test(),
    Connect(ConnectPacket),
    ConnAck(ConnAckPacket),
}

impl ControlPacket {
    pub fn new(packet_id: u8) -> ControlPacket {
        match packet_id {
            1 => Connect(ConnectPacket::default()),
            2 => ConnAck(ConnAckPacket::default()),
            _ => panic!("Invalid packet id!"),
        }
    }
}
