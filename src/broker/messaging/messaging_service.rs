use std::collections::{HashMap, HashSet};

use log::warn;

use crate::broker::messaging::subscriptions_repository::SubscriptionsRepository;
use crate::broker::session::session_repository::SessionRepository;
use crate::broker::session::{InMemorySessionRepository, Session, SessionService};
use crate::mqtt::events::ServerEvent;
use crate::mqtt::message::Message;
use crate::mqtt::packets::suback::SubAckReturnCode;
use crate::mqtt::packets::ControlPacket::Publish;
use crate::mqtt::packets::{ClientId, PublishPacket};
use crate::mqtt::subscription::Subscription;

pub struct MessagingService {
    sessions: InMemorySessionRepository,
    subscriptions: SubscriptionsRepository,
}

impl MessagingService {
    pub fn new() -> Self {
        MessagingService {
            sessions: InMemorySessionRepository::default(),
            subscriptions: SubscriptionsRepository::new(),
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

    pub fn connection_lost(&mut self, client_id: &ClientId) -> Option<Session> {
        self.subscriptions.connections_lost(client_id);

        self.sessions.delete(client_id)
    }

    pub fn disconnect(&mut self, client_id: &ClientId) -> Option<Session> {
        self.subscriptions.disconnected(client_id);

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
        self.subscriptions.subscribe(client_id, subscription)
    }

    pub fn unsubscribe(&mut self, client_id: &ClientId, topics: &Vec<String>) {
        self.subscriptions.unsubscribe(client_id, topics);
    }

    pub async fn publish(&self, message: &Message, publish: &PublishPacket) {
        if let Some(client_ids) = self.subscriptions.subscribed_clients(&message.topic) {
            for c in &client_ids {
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
