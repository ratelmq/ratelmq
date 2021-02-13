use crate::mqtt::packets::QoS;
use bytes::BytesMut;

#[derive(Debug, PartialEq, Clone, Default)]
pub struct PublishPacket {
    pub dup: bool,
    pub qos: QoS,
    pub retain: bool,
    pub packet_id: Option<u16>,

    pub topic: String,
    pub body: BytesMut,
}

impl PublishPacket {
    pub fn new(
        topic: String,
        body: BytesMut,
        qos: QoS,
        retain: bool,
        packet_id: Option<u16>,
        dup: bool,
    ) -> Self {
        PublishPacket {
            dup,
            qos,
            retain,
            packet_id,
            topic,
            body,
        }
    }
}
