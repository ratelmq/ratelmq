use log::{debug, info, trace, warn};

use crate::broker::messaging::subscriptions_repository::SubscriptionsRepository;
use crate::broker::session::session_repository::SessionRepository;
use crate::broker::session::{InMemorySessionRepository, Session};
use crate::mqtt::events::ServerEvent;
use crate::mqtt::message::Message;
use crate::mqtt::packets::suback::SubAckReturnCode;
use crate::mqtt::packets::ControlPacket::Publish;
use crate::mqtt::packets::{ClientId, PublishPacket};
use crate::mqtt::subscription::Subscription;
use chrono::Utc;
use std::collections::hash_map::Iter;
use tokio::sync::{mpsc, oneshot};

pub type MessagingTx = mpsc::Sender<MessagingOperation>;
pub type MessagingRx = mpsc::Receiver<MessagingOperation>;

type Responder<T> = oneshot::Sender<T>;

#[derive(Debug)]
pub enum MessagingOperation {
    SessionExists {
        client_id: ClientId,
        resp: Responder<bool>,
    },
    SessionInsert {
        session: Session,
        resp: Responder<()>,
    },
    SessionGet {
        client_id: ClientId,
        resp: Responder<Option<Session>>,
    },
    SessionGetExpiredKeepAlive {
        resp: Responder<Vec<Session>>,
    },
    SessionCount {
        resp: Responder<usize>,
    },

    ConnectionLost {
        client_id: ClientId,
        resp: Responder<Option<Session>>,
    },
    ConnectionDisconnected {
        client_id: ClientId,
        resp: Responder<Option<Session>>,
    },

    Subscribe {
        client_id: ClientId,
        subscription: Subscription,
        resp: Responder<SubAckReturnCode>,
    },
    Unsubscribe {
        client_id: ClientId,
        topics: Vec<String>,
        resp: Responder<()>,
    },
    Publish {
        message: Message,
        publish: PublishPacket,
        resp: Responder<()>,
    },
    SendersToPublish {
        topic: String,
        resp: Responder<Vec<mpsc::Sender<ServerEvent>>>,
    },
}

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

    pub async fn run(mut self, mut rx: mpsc::Receiver<MessagingOperation>) {
        debug!("Started Messaging Manager");
        while let Some(op) = rx.recv().await {
            match op {
                MessagingOperation::SessionExists { client_id, resp } => {
                    let result = self.session_exists(&client_id);
                    let _ = resp.send(result);
                }
                MessagingOperation::SessionInsert { session, resp } => {
                    self.session_insert(session);
                    let _ = resp.send(());
                }
                MessagingOperation::SessionGet { client_id, resp } => {
                    // let result = self.session_get(&client_id);
                    let _ = resp.send(None);
                }
                MessagingOperation::SessionGetExpiredKeepAlive { resp } => {
                    // let result = self.session_get_keep_alive_expired();
                    let _ = resp.send(Vec::new());
                }
                MessagingOperation::SessionCount { resp } => {
                    let result = self.session_count();
                    let _ = resp.send(result);
                }
                MessagingOperation::ConnectionLost { client_id, resp } => {
                    let result = self.connection_lost(&client_id);
                    let _ = resp.send(result);
                }
                MessagingOperation::ConnectionDisconnected { client_id, resp } => {
                    let result = self.disconnect(&client_id);
                    let _ = resp.send(result);
                }
                MessagingOperation::Subscribe { client_id, subscription, resp } => {
                    let result = self.subscribe(&client_id, &subscription);
                    let _ = resp.send(result);
                }
                MessagingOperation::Unsubscribe { client_id, topics, resp } => {
                    self.unsubscribe(&client_id, &topics);
                    let _ = resp.send(());
                }
                MessagingOperation::Publish { message, publish, resp } => {
                    self.publish(&message, &publish);
                    let _ = resp.send(());
                }

                MessagingOperation::SendersToPublish { topic, resp } => {
                    let result = self.senders_to_publish(&topic);
                    let _ = resp.send(result);
                }
            }
        }
        debug!("Stopped Messaging Manager");
    }

    pub fn session_exists(&self, client_id: &ClientId) -> bool {
        self.sessions.exists(client_id)
    }

    pub fn session_insert(&mut self, session: Session) {
        // trace!()
        self.sessions.insert(session)
    }

    pub fn session_get(&self, client_id: &ClientId) -> Option<&Session> {
        self.sessions.get(client_id)
    }

    pub fn session_get_mut(&mut self, client_id: &ClientId) -> Option<&mut Session> {
        self.sessions.get_mut(client_id)
    }

    pub fn session_get_keep_alive_expired(&self) -> Vec<&Session> {
        let now = Utc::now();

        self.sessions
            .iter()
            .filter_map(
                |(client_id, session)| match session.is_keep_alive_expired(&now) {
                    true => Some(session),
                    false => None,
                },
            )
            .collect()
    }

    pub fn session_count(&self) -> usize {
        self.sessions.count()
    }

    pub fn connection_lost(&mut self, client_id: &ClientId) -> Option<Session> {
        self.subscriptions.connections_lost(client_id);

        self.sessions.delete(client_id)
    }

    pub fn disconnect(&mut self, client_id: &ClientId) -> Option<Session> {
        self.subscriptions.disconnected(client_id);

        self.sessions.delete(client_id)
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

    pub fn senders_to_publish(&self, topic: &String) -> Vec<mpsc::Sender<ServerEvent>> {
        let mut senders = Vec::new();

        if let Some(client_ids) = self.subscriptions.subscribed_clients(topic) {
            for c in &client_ids {
                match self.sessions.get(c) {
                    Some(session) => {
                        senders.push(session.sender().clone());
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

        senders
    }
}
