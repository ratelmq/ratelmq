#[derive(Debug, PartialEq, Clone, Default)]
pub struct UnSubAckPacket {
    pub packet_id: u16,
}

impl UnSubAckPacket {
    pub fn new(packet_id: u16) -> Self {
        UnSubAckPacket { packet_id }
    }
}
