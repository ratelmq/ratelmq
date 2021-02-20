use crate::mqtt::events::ServerEvent;
use crate::mqtt::packets::ClientId;
use std::net::IpAddr;
use tokio::sync::mpsc::Sender;

pub struct Session {
    client_id: ClientId,
    ip: IpAddr,
    persistent: bool,
    sender: Sender<ServerEvent>,
}

impl Session {
    pub fn new(
        client_id: ClientId,
        ip: IpAddr,
        persistent: bool,
        sender: Sender<ServerEvent>,
    ) -> Self {
        Session {
            client_id,
            ip,
            persistent,
            sender,
        }
    }

    pub fn sender(&self) -> &Sender<ServerEvent> {
        &self.sender
    }

    pub fn client_id(&self) -> &ClientId {
        &self.client_id
    }
}
