use std::collections::HashMap;

use crate::broker::session::session_repository::SessionRepository;
use crate::broker::session::{InMemorySessionRepository, Session, SessionService};
use crate::mqtt::events::ServerEvent;
use crate::mqtt::message::Message;
use crate::mqtt::packets::suback::SubAckReturnCode;
use crate::mqtt::packets::ControlPacket::Publish;
use crate::mqtt::packets::{ClientId, PublishPacket};
use crate::mqtt::subscription::Subscription;
use log::warn;

type Subs = HashMap<String, Vec<ClientId>>;

#[derive(Default)]
pub struct MessagingService {
    sessions: InMemorySessionRepository,
    subscriptions: Subs,
}

impl MessagingService {
    pub fn new() -> Self {
        MessagingService {
            sessions: InMemorySessionRepository::default(),
            subscriptions: HashMap::new(),
        }
    }

    pub fn session_exists(&self, client_id: &ClientId) -> bool {
        self.sessions.exists(client_id)
    }

    pub fn session_insert(&mut self, session: Session) {
        self.sessions.insert(session)
    }

    pub fn session_get(&self, client_id: &ClientId) -> Option<&Session> {
        self.sessions.get(client_id)
    }

    pub fn session_delete(&mut self, client_id: &ClientId) -> Option<Session> {
        self.sessions.delete(client_id)
    }

    pub fn session_count(&self) -> usize {
        self.sessions.count()
    }

    pub fn subscribe(
        &mut self,
        client_id: &ClientId,
        subscription: &Subscription,
    ) -> SubAckReturnCode {
        self.subscriptions
            .entry(subscription.topic().to_string())
            .or_insert(Vec::new())
            .push(client_id.clone());

        SubAckReturnCode::SuccessQoS0
    }

    pub fn unsubscribe(&self, client_id: &ClientId, topics: &Vec<String>) {}

    pub async fn publish(&self, message: &Message, publish: &PublishPacket) {
        if let Some(client_ids) = self.subscriptions.get(&message.topic) {
            for c in client_ids {
                match self.sessions.get(c) {
                    Some(session) => {
                        let event = ServerEvent::ControlPacket(Publish(publish.clone()));
                        session.sender().send(event).await.unwrap();
                    }
                    None => {
                        warn!(
                            "Tried to send message, but session for client {:?} not found",
                            c
                        );
                    }
                }
            }
        }
    }
}
