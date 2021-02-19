use crate::mqtt::packets::{ClientId, ProtocolVersion};

use crate::mqtt::message::Message;

#[derive(Debug, PartialEq, Clone, Default)]
pub struct ConnectPacket {
    pub version: ProtocolVersion,
    pub client_id: ClientId,
    pub keep_alive_seconds: u16,
    pub clean_session: bool,

    pub will_message: Option<Message>,

    pub user_name: Option<String>,
    pub password: Option<String>,
}

impl ConnectPacket {
    pub fn new(
        version: ProtocolVersion,
        client_id: String,
        keep_alive_seconds: u16,
        clean_session: bool,
        will_message: Option<Message>,
        user_name: Option<String>,
        password: Option<String>,
    ) -> Self {
        ConnectPacket {
            version,
            client_id,
            keep_alive_seconds,
            clean_session,
            will_message,
            user_name,
            password,
        }
    }
}
