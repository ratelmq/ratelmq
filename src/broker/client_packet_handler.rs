use std::net::SocketAddr;

use chrono::Utc;
use log::{debug, error, info, trace};
use tokio::select;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::{broadcast, mpsc, oneshot};
use uuid::Uuid;

use crate::broker::authentication::{FileIdentityManager, IdentityProvider};
use crate::broker::messaging::{MessagingOperation, MessagingService, MessagingTx};
use crate::broker::session::Session;
use crate::mqtt::events::{ClientEvent, ServerEvent};
use crate::mqtt::packets::connack::ConnAckReturnCode;
use crate::mqtt::packets::suback::SubAckPacket;
use crate::mqtt::packets::subscribe::SubscribePacket;
use crate::mqtt::packets::unsuback::UnSubAckPacket;
use crate::mqtt::packets::unsubscribe::UnsubscribePacket;
use crate::mqtt::packets::ControlPacket::{ConnAck, PingResp, Publish, SubAck, UnsubAck};
use crate::mqtt::packets::*;
use crate::settings::Settings;

pub struct ClientPacketHandler {
    rx: mpsc::Receiver<ClientEvent>,
    ctrl_c_rx: broadcast::Receiver<()>,
    // sessions: SessionService<InMemorySessionRepository>,
    // messaging: MessagingServiceSync,
    messaging_tx: MessagingTx,
    identity_provider: Box<dyn IdentityProvider + Send + Sync>,
}

impl ClientPacketHandler {
    pub fn new(
        rx: Receiver<ClientEvent>,
        ctrl_c_rx: broadcast::Receiver<()>,
        settings: &Settings,
        // messaging: MessagingServiceSync,
        messaging_tx: MessagingTx,
    ) -> ClientPacketHandler {
        let identity_provider = Box::new(
            FileIdentityManager::new(settings.authentication.password_file.as_str()).unwrap(),
        );

        ClientPacketHandler {
            rx,
            ctrl_c_rx,
            // sessions: SessionService::default(),
            // messaging,
            messaging_tx,
            identity_provider,
        }
    }

    pub async fn run(mut self) {
        loop {
            select! {
                _ = self.ctrl_c_rx.recv() => {
                    debug!("Stopping manager...");
                    break;
                }
                 maybe_event = self.rx.recv() => {
                    if let Some(event) = maybe_event {
                        trace!("Got event {:?}", &event);

                        match event {

                            ClientEvent::Connected(c, address, tx) => self.on_connect(tx, c, address).await,
                            ClientEvent::ControlPacket(client_id, packet, tx) => {
                                self.on_packet(client_id, packet, tx).await;
                            }
                            ClientEvent::Disconnected(_client_id) => {}
                            ClientEvent::ConnectionLost(client_id) => {
                                self.on_connection_lost(client_id).await;
                            }
                        }
                    }
                 }
            }
        }

        debug!("Stopped Manager");
    }

    async fn on_packet(
        &mut self,
        client_id: String,
        packet: ControlPacket,
        tx: Sender<ServerEvent>,
    ) {
        trace!("Got packet {:?}", packet);

        match packet {
            // ControlPacket::Connect(c) => {
            //     self.on_connect(action.response, c, &mut sessions).await
            // }
            // ControlPacket::ConnAck(_) => {}
            ControlPacket::Publish(p) => self.on_publish(tx, p, client_id).await,
            // ControlPacket::PubAck(_) => {}
            // ControlPacket::PubRec(_) => {}
            // ControlPacket::PubRel(_) => {}
            // ControlPacket::PubComp(_) => {}
            ControlPacket::Subscribe(p) => self.on_subscribe(tx, p, &client_id).await,
            // ControlPacket::SubAck(_) => {}
            ControlPacket::Unsubscribe(p) => self.on_unsubscribe(tx, p, &client_id).await,
            // ControlPacket::UnsubAck(_) => {}
            ControlPacket::PingReq => self.on_ping_req(tx, &client_id).await,
            // ControlPacket::PingResp() => {}
            ControlPacket::Disconnect(_) => self.on_disconnect(client_id).await,
            _ => error!("Packet {} not supported", &packet),
        };
    }

    async fn on_connect(
        &mut self,
        sender: Sender<ServerEvent>,
        packet: ConnectPacket,
        address: SocketAddr,
    ) {
        let client_id = packet.client_id;
        debug!("New client {:?} connected", &client_id);

        if let Some(user_name) = packet.user_name {
            let password = packet.password.unwrap();
            if let Err(e) = self.identity_provider.authenticate(&user_name, &password) {
                info!("Client {} authentication error: {:?}", &user_name, &e);

                let conn_ack = ConnAckPacket::new(false, ConnAckReturnCode::NotAuthorized);
                sender
                    .send(ServerEvent::ControlPacket(ConnAck(conn_ack)))
                    .await
                    .unwrap();
                sender.send(ServerEvent::Disconnect).await.unwrap();
                return;
            }
        };

        let session_present = {
            let (tx, rx) = oneshot::channel();
            let op = MessagingOperation::SessionGet { client_id: client_id.clone(), resp: tx };

            self.messaging_tx.send(op).await.unwrap();
            let maybe_session = rx.await.unwrap();

            if let Some(session) = maybe_session {
                // session.set_last_activity(Utc::now());
                true
            } else {
                let session = Session::new(
                    client_id,
                    address.ip(),
                    !packet.clean_session,
                    sender.clone(),
                    packet.keep_alive_seconds,
                    Utc::now(),
                );
                let (insert_tx, insert_rx) = oneshot::channel();
                let op = MessagingOperation::SessionInsert { session, resp: insert_tx };
                self.messaging_tx.send(op).await.unwrap();
                let insert_result = insert_rx.await.unwrap();
                false
            }
        };

        let conn_ack = ConnAckPacket::new(session_present, ConnAckReturnCode::Accepted);

        sender
            .send(ServerEvent::ControlPacket(ConnAck(conn_ack)))
            .await
            .unwrap();

        // debug!(
        //     "Active sessions count: {:?}",
        //     self.messaging.session_count()
        // );
    }

    async fn on_disconnect(&mut self, client_id: ClientId) {
        debug!("Client {:?} disconnected", &client_id);

        let (tx, rx) = oneshot::channel();
        let op = MessagingOperation::ConnectionDisconnected { client_id, resp: tx };

        self.messaging_tx.send(op).await.unwrap();
        let maybe_session = rx.await.unwrap();

        // debug!(
        //     "Active sessions count: {:?}",
        //     messaging.session_count()
        // );
    }

    async fn on_connection_lost(&mut self, client_id: ClientId) {
        info!("Client {:?} disconnected unexpectedly", &client_id);

        let (tx, rx) = oneshot::channel();
        let op = MessagingOperation::ConnectionLost { client_id, resp: tx };

        self.messaging_tx.send(op).await.unwrap();
        let maybe_session = rx.await.unwrap();

        // debug!(
        //     "Active sessions count: {:?}",
        //     self.messaging.session_count()
        // );
    }

    async fn on_publish(
        &self,
        _sender: Sender<ServerEvent>,
        publish: PublishPacket,
        client_id: ClientId,
    ) {
        debug!(
            "Client {:?} published message on topic {:?}",
            &client_id, &publish.message.topic
        );

        let (tx, rx) = oneshot::channel();
        let op = MessagingOperation::SendersToPublish { topic: publish.message.topic.clone(), resp: tx };

        self.messaging_tx.send(op).await.unwrap();
        let senders_to_publish = rx.await.unwrap();

        for sender in senders_to_publish {
            let event = ServerEvent::ControlPacket(Publish(publish.clone()));
            sender.send(event).await.unwrap();
        }
    }

    async fn on_subscribe(
        &mut self,
        sender: Sender<ServerEvent>,
        subscribe: SubscribePacket,
        client_id: &ClientId,
    ) {
        debug!("Client {:?} subscribed to topics {:?}", client_id, &subscribe.subscriptions);

        let mut return_codes = Vec::new();

        for subscription in subscribe.subscriptions {
            // each subscription request must be handled as a separate subscribe packet

            let (tx, rx) = oneshot::channel();
            let op = MessagingOperation::Subscribe { client_id: client_id.clone(), subscription, resp: tx };

            self.messaging_tx.send(op).await.unwrap();
            let return_code = rx.await.unwrap();

            return_codes.push(return_code);
        }

        let sub_ack = SubAckPacket::new(subscribe.packet_id, return_codes);
        sender
            .send(ServerEvent::ControlPacket(SubAck(sub_ack)))
            .await
            .unwrap();
    }

    async fn on_unsubscribe(
        &mut self,
        sender: Sender<ServerEvent>,
        unsubscribe: UnsubscribePacket,
        client_id: &ClientId,
    ) {
        debug!("Client {:?} unsubscribed from topics {:?}", client_id, &unsubscribe.topics);

        let (tx, rx) = oneshot::channel();
        let op = MessagingOperation::Unsubscribe { client_id: client_id.clone(), topics: unsubscribe.topics, resp: tx };

        self.messaging_tx.send(op).await.unwrap();
        let _ = rx.await.unwrap();

        let unsub_ack = UnSubAckPacket::new(unsubscribe.packet_id);
        sender
            .send(ServerEvent::ControlPacket(UnsubAck(unsub_ack)))
            .await
            .unwrap();
    }

    async fn on_ping_req(&mut self, sender: Sender<ServerEvent>, client_id: &ClientId) {
        // let session_present = {
        //     let (tx, rx) = oneshot::channel();
        //     let op = MessagingOperation::SessionGet { client_id: client_id.clone(), resp: tx };
        //
        //     self.messaging_tx.send(op).await.unwrap();
        //     let maybe_session = rx.await.unwrap();
        //
        //     info!("Client {:?}, Session: {:?}", client_id, &maybe_session);
        //     if let Some(session) = maybe_session {
        //         // session.set_last_activity(Utc::now());
        //         true
        //     } else {
        //         false
        //     }
        // };

        let session_present = {
            let (tx, rx) = oneshot::channel();
            let op = MessagingOperation::SessionExists { client_id: client_id.clone(), resp: tx };

            self.messaging_tx.send(op).await.unwrap();
            rx.await.unwrap()
        };

        if session_present {
            sender
                .send(ServerEvent::ControlPacket(PingResp))
                .await
                .unwrap();
        } else {
            error!(
                "Received PING from not existing session with client id {}",
                client_id
            );
            sender.send(ServerEvent::Disconnect).await.unwrap();
        }
    }
}
