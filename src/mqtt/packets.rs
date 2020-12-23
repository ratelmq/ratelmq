use num_enum::FromPrimitive;

use crate::mqtt::packets::ConnAckReturnCode::Accepted;
use crate::mqtt::packets::ProtocolVersion::Mqtt3;
use crate::mqtt::packets::QoS::AtMostOnce;

pub const FIXED_HEADER_MAX_SIZE: usize = 1 + 4;

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

#[derive(Debug, PartialEq, Clone, Default)]
pub struct ConnectPacket {
    // header
    pub version: ProtocolVersion,
    pub clean_session: bool,
    pub will_flag: bool,
    pub will_qos: QoS,
    pub will_retain: bool,
    pub keep_alive_seconds: u16,

    // payload
    pub client_identifier: &'static str,
    pub will_topic: Option<&'static str>,
    pub will_message: Option<&'static str>,
    pub user_name: Option<&'static str>,
    pub password: Option<&'static str>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum ConnAckReturnCode {
    Accepted = 0x00,
    UnacceptableProtocolVersion,
    IdentifierRejected,
    ServerUnavailable,
    BadUserNameOrPassword,
    NotAuthorized,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ConnAckPacket {
    pub session_present: bool,
    pub return_code: ConnAckReturnCode,
}
