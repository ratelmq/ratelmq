use crate::mqtt::packets::QoS;
use bytes::BytesMut;

#[derive(Debug, PartialEq, Clone, Default)]
pub struct Message {
    pub topic: String,
    pub payload: BytesMut,
    pub qos: QoS,
    pub retain: bool,
}
