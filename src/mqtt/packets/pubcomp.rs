#[derive(Debug, PartialEq, Clone, Default)]
pub struct PubCompPacket {
    pub packet_id: u16,
}

impl PubCompPacket {
    pub fn new(packet_id: u16) -> Self {
        PubCompPacket { packet_id }
    }
}
