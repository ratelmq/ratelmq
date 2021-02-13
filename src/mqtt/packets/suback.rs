use crate::mqtt::packets::suback::SubAckReturnCode::Failure;

#[derive(Debug, PartialEq, Clone)]
pub enum SubAckReturnCode {
    SuccessQoS0 = 0x00,
    SuccessQoS1 = 0x01,
    SuccessQoS2 = 0x02,
    Failure = 0x80,
}

impl Default for SubAckReturnCode {
    fn default() -> Self {
        Failure
    }
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct SubAckPacket {
    pub packet_id: u16,

    pub return_codes: Vec<SubAckReturnCode>,
}

impl SubAckPacket {
    pub fn new(packet_id: u16, return_codes: Vec<SubAckReturnCode>) -> Self {
        SubAckPacket {
            packet_id,
            return_codes,
        }
    }
}
