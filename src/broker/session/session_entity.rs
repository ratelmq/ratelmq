use crate::mqtt::events::ServerEvent;
use crate::mqtt::packets::ClientId;
use chrono::{DateTime, Duration, Utc};
use std::net::IpAddr;
use tokio::sync::mpsc::Sender;

#[derive(Debug)]
pub struct Session {
    client_id: ClientId,
    ip: IpAddr,
    persistent: bool,
    sender: Sender<ServerEvent>,
    pub keep_alive_seconds: u16,
    last_activity: DateTime<Utc>,
}

impl Session {
    pub fn new(
        client_id: ClientId,
        ip: IpAddr,
        persistent: bool,
        sender: Sender<ServerEvent>,
        keep_alive_seconds: u16,
        last_activity: DateTime<Utc>,
    ) -> Self {
        Session {
            client_id,
            ip,
            persistent,
            sender,
            keep_alive_seconds,
            last_activity,
        }
    }

    pub fn sender(&self) -> &Sender<ServerEvent> {
        &self.sender
    }

    pub fn client_id(&self) -> &ClientId {
        &self.client_id
    }

    pub fn set_last_activity(&mut self, last_activity: DateTime<Utc>) {
        self.last_activity = last_activity;
    }

    pub fn last_activity(&self) -> &DateTime<Utc> {
        &self.last_activity
    }

    pub fn is_keep_alive_expired(&self, now: &DateTime<Utc>) -> bool {
        if self.keep_alive_seconds == 0 {
            return false;
        }

        let leeway_seconds = (self.keep_alive_seconds as f32 * 1.5) as i64;
        let keep_alive_expires_at = self.last_activity + Duration::seconds(leeway_seconds);

        &keep_alive_expires_at <= now
    }
}
