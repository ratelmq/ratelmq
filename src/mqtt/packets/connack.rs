#[derive(Debug, PartialEq, Clone)]
pub enum ConnAckReturnCode {
    Accepted = 0x00,
    UnacceptableProtocolVersion,
    IdentifierRejected,
    ServerUnavailable,
    BadUserNameOrPassword,
    NotAuthorized,
}

impl Default for ConnAckReturnCode {
    fn default() -> Self {
        ConnAckReturnCode::Accepted
    }
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct ConnAckPacket {
    pub session_present: bool,
    pub return_code: ConnAckReturnCode,
}

impl ConnAckPacket {
    pub fn new(session_present: bool, return_code: ConnAckReturnCode) -> Self {
        ConnAckPacket {
            session_present,
            return_code,
        }
    }
}
