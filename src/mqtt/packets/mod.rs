pub use crate::mqtt::packets::connack::ConnAckPacket;
pub use crate::mqtt::packets::connect::ConnectPacket;
pub use crate::mqtt::packets::disconnect::DisconnectPacket;
use crate::mqtt::packets::ping_req::PingReqPacket;
use crate::mqtt::packets::ping_resp::PingRespPacket;
use crate::mqtt::packets::puback::PubAckPacket;
pub use crate::mqtt::packets::publish::PublishPacket;
use crate::mqtt::packets::suback::SubAckPacket;
use crate::mqtt::packets::subscribe::SubscribePacket;
use crate::mqtt::packets::unsuback::UnSubAckPacket;
use crate::mqtt::packets::unsubscribe::UnsubscribePacket;
use crate::mqtt::packets::ControlPacket::{
    ConnAck, Connect, Disconnect, PingReq, PingResp, PubAck, Publish, SubAck, Subscribe, UnsubAck,
    Unsubscribe,
};
use crate::mqtt::packets::ProtocolVersion::Mqtt3;
use crate::mqtt::packets::QoS::{AtLeastOnce, AtMostOnce, ExactlyOnce};

pub mod connack;
pub mod connect;
pub mod disconnect;
pub mod ping_req;
pub mod ping_resp;
pub mod puback;
pub mod publish;
pub mod suback;
pub mod subscribe;
pub mod unsuback;
pub mod unsubscribe;

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
            1 => Connect(ConnectPacket::default()),
            2 => ConnAck(ConnAckPacket::default()),
            3 => Publish(PublishPacket::default()),
            4 => PubAck(PubAckPacket::default()),
            // 5 => PubRec
            // 6 => PubRel
            // 7 => PubComp
            8 => Subscribe(SubscribePacket::default()),
            9 => SubAck(SubAckPacket::default()),
            10 => Unsubscribe(UnsubscribePacket::default()),
            11 => UnsubAck(UnSubAckPacket::default()),
            12 => PingReq(PingReqPacket::default()),
            13 => PingResp(PingRespPacket::default()),
            14 => Disconnect(DisconnectPacket::default()),
            _ => panic!("Invalid packet id!"),
        }
    }
}
