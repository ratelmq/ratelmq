#[derive(Debug, PartialEq, Clone, Default)]
pub struct PubAckPacket {
    pub packet_id: u16,
}

impl PubAckPacket {
    pub fn new(packet_id: u16) -> Self {
        PubAckPacket { packet_id }
    }
}
