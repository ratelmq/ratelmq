use crate::mqtt::message::Message;
use crate::mqtt::packets::QoS;
use bytes::BytesMut;

#[derive(Debug, PartialEq, Clone, Default)]
pub struct PublishPacket {
    pub packet_id: Option<u16>,
    pub dup: bool,

    pub message: Message,
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
            packet_id,
            dup,
            message: Message {
                qos,
                retain,
                topic,
                payload: body,
            },
        }
    }
}
