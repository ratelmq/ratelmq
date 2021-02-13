#[derive(Debug, PartialEq, Clone, Default)]
pub struct PubRelPacket {
    pub packet_id: u16,
}

impl PubRelPacket {
    pub fn new(packet_id: u16) -> Self {
        PubRelPacket { packet_id }
    }
}
