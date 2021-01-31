pub use crate::mqtt::packets::connack::ConnAckPacket;
pub use crate::mqtt::packets::connect::ConnectPacket;
pub use crate::mqtt::packets::disconnect::DisconnectPacket;
use crate::mqtt::packets::ping_req::PingReqPacket;
use crate::mqtt::packets::ping_resp::PingRespPacket;
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
const PACKET_TYPE_CONNECT: u8 = 1;
const PACKET_TYPE_CONN_ACK: u8 = 2;
const PACKET_TYPE_PUBLISH: u8 = 3;
const PACKET_TYPE_PUB_ACK: u8 = 4;
const PACKET_TYPE_PUB_REC: u8 = 5;
const PACKET_TYPE_PUB_REL: u8 = 6;
const PACKET_TYPE_PUB_COMP: u8 = 7;
const PACKET_TYPE_SUBSCRIBE: u8 = 8;
const PACKET_TYPE_SUB_ACK: u8 = 9;
const PACKET_TYPE_UNSUBSCRIBE: u8 = 10;
const PACKET_TYPE_UNSUB_ACK: u8 = 11;
const PACKET_TYPE_PING_REQ: u8 = 12;
const PACKET_TYPE_PING_RESP: u8 = 13;
const PACKET_TYPE_DISCONNECT: u8 = 14;
// const PACKET_TYPE_RESERVED15: u8 = 15;

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
    PingReq(PingReqPacket),
    PingResp(PingRespPacket),
    Disconnect(DisconnectPacket),
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
            PACKET_TYPE_PING_REQ => PingReq(PingReqPacket::default()),
            PACKET_TYPE_PING_RESP => PingResp(PingRespPacket::default()),
            PACKET_TYPE_DISCONNECT => Disconnect(DisconnectPacket::default()),
            _ => panic!("Invalid packet id!"),
        }
    }
}
