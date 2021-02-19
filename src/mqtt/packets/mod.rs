pub use crate::mqtt::packets::connack::ConnAckPacket;
pub use crate::mqtt::packets::connect::ConnectPacket;
pub use crate::mqtt::packets::disconnect::DisconnectPacket;
use crate::mqtt::packets::puback::PubAckPacket;
use crate::mqtt::packets::pubcomp::PubCompPacket;
pub use crate::mqtt::packets::publish::PublishPacket;
use crate::mqtt::packets::pubrec::PubRecPacket;
use crate::mqtt::packets::pubrel::PubRelPacket;
use crate::mqtt::packets::suback::SubAckPacket;
use crate::mqtt::packets::subscribe::SubscribePacket;
use crate::mqtt::packets::unsuback::UnSubAckPacket;
use crate::mqtt::packets::unsubscribe::UnsubscribePacket;
use crate::mqtt::packets::ControlPacket::{
    ConnAck, Connect, Disconnect, PingReq, PingResp, PubAck, PubComp, PubRec, PubRel, Publish,
    SubAck, Subscribe, UnsubAck, Unsubscribe,
};
use crate::mqtt::packets::ProtocolVersion::Mqtt3;
use crate::mqtt::packets::QoS::{AtLeastOnce, AtMostOnce, ExactlyOnce};
use std::fmt;
use std::fmt::{Display, Formatter};

pub mod connack;
pub mod connect;
pub mod disconnect;
pub mod ping_req;
pub mod ping_resp;
pub mod puback;
pub mod pubcomp;
pub mod publish;
pub mod pubrec;
pub mod pubrel;
pub mod suback;
pub mod subscribe;
pub mod unsuback;
pub mod unsubscribe;

// const PACKET_TYPE_RESERVED0: u8 = 0;
pub const PACKET_TYPE_CONNECT: u8 = 1;
pub const PACKET_TYPE_CONN_ACK: u8 = 2;
pub const PACKET_TYPE_PUBLISH: u8 = 3;
pub const PACKET_TYPE_PUB_ACK: u8 = 4;
pub const PACKET_TYPE_PUB_REC: u8 = 5;
pub const PACKET_TYPE_PUB_REL: u8 = 6;
pub const PACKET_TYPE_PUB_COMP: u8 = 7;
pub const PACKET_TYPE_SUBSCRIBE: u8 = 8;
pub const PACKET_TYPE_SUB_ACK: u8 = 9;
pub const PACKET_TYPE_UNSUBSCRIBE: u8 = 10;
pub const PACKET_TYPE_UNSUB_ACK: u8 = 11;
pub const PACKET_TYPE_PING_REQ: u8 = 12;
pub const PACKET_TYPE_PING_RESP: u8 = 13;
pub const PACKET_TYPE_DISCONNECT: u8 = 14;
// const PACKET_TYPE_RESERVED15: u8 = 15;

pub type ClientId = String;

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

#[derive(Debug, PartialEq, PartialOrd, Clone, Copy)]
pub enum QoS {
    AtMostOnce = 0,
    AtLeastOnce = 1,
    ExactlyOnce = 2,
}

impl Display for QoS {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match *self {
            AtMostOnce => write!(f, "0 (at most once)"),
            AtLeastOnce => write!(f, "1 (at least once)"),
            ExactlyOnce => write!(f, "2 (exactly once)"),
        }
    }
}

impl QoS {
    pub fn from_bits(bits: u8) -> QoS {
        match bits {
            0 => AtMostOnce,
            1 => AtLeastOnce,
            2 => ExactlyOnce,
            _ => panic!("Invalid QoS"),
        }
    }
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
    PubAck(PubAckPacket),
    PubRec(PubRecPacket),
    PubRel(PubRelPacket),
    PubComp(PubCompPacket),
    Subscribe(SubscribePacket),
    SubAck(SubAckPacket),
    Unsubscribe(UnsubscribePacket),
    UnsubAck(UnSubAckPacket),
    PingReq,
    PingResp,
    Disconnect(DisconnectPacket),
}

impl Display for ControlPacket {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            Connect(_) => write!(f, "CONNECT"),
            ConnAck(_) => write!(f, "CONNACK"),
            Publish(_) => write!(f, "PUBLISH"),
            PubAck(_) => write!(f, "PUBACK"),
            PubRec(_) => write!(f, "PUBREC"),
            PubRel(_) => write!(f, "PUBREL"),
            PubComp(_) => write!(f, "PUBCOMP"),
            Subscribe(_) => write!(f, "SUBSCRIBE"),
            SubAck(_) => write!(f, "SUBACK"),
            Unsubscribe(_) => write!(f, "UNSUBSCRIBE"),
            UnsubAck(_) => write!(f, "UNSUBACK"),
            PingReq => write!(f, "PINGREQ"),
            PingResp => write!(f, "PINGRESP"),
            Disconnect(_) => write!(f, "DISCONNECT"),
        }
    }
}

impl ControlPacket {
    pub fn new(packet_id: u8) -> ControlPacket {
        match packet_id {
            PACKET_TYPE_CONNECT => Connect(ConnectPacket::default()),
            PACKET_TYPE_CONN_ACK => ConnAck(ConnAckPacket::default()),
            PACKET_TYPE_PUBLISH => Publish(PublishPacket::default()),
            PACKET_TYPE_PUB_ACK => PubAck(PubAckPacket::default()),
            PACKET_TYPE_PUB_REC => PubRec(PubRecPacket::default()),
            PACKET_TYPE_PUB_REL => PubRel(PubRelPacket::default()),
            PACKET_TYPE_PUB_COMP => PubComp(PubCompPacket::default()),
            PACKET_TYPE_SUBSCRIBE => Subscribe(SubscribePacket::default()),
            PACKET_TYPE_SUB_ACK => SubAck(SubAckPacket::default()),
            PACKET_TYPE_UNSUBSCRIBE => Unsubscribe(UnsubscribePacket::default()),
            PACKET_TYPE_UNSUB_ACK => UnsubAck(UnSubAckPacket::default()),
            PACKET_TYPE_PING_REQ => PingReq,
            PACKET_TYPE_PING_RESP => PingResp,
            PACKET_TYPE_DISCONNECT => Disconnect(DisconnectPacket::default()),
            _ => panic!("Invalid packet id!"),
        }
    }
}
