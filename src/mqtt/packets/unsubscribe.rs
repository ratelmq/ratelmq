#[derive(Debug, PartialEq, Clone, Default)]
pub struct UnsubscribePacket {
    pub packet_id: u16,

    pub topics: Vec<String>,
}

impl UnsubscribePacket {
    pub fn new(packet_id: u16, topics: Vec<String>) -> Self {
        UnsubscribePacket { packet_id, topics }
    }
}
