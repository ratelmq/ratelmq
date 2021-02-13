#[derive(Debug, PartialEq, Clone, Default)]
pub struct PubRecPacket {
    pub packet_id: u16,
}

impl PubRecPacket {
    pub fn new(packet_id: u16) -> Self {
        PubRecPacket { packet_id }
    }
}
