use std::net::SocketAddr;

use chrono::Utc;
use log::{debug, error, info, trace};
use tokio::select;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::{broadcast, mpsc};

use crate::broker::authentication::{FileIdentityManager, IdentityProvider};
use crate::broker::messaging::{MessagingService, MessagingServiceSync};
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
    messaging: MessagingServiceSync,
    identity_provider: Box<dyn IdentityProvider + Send + Sync>,
}

impl ClientPacketHandler {
    pub fn new(
        rx: Receiver<ClientEvent>,
        ctrl_c_rx: broadcast::Receiver<()>,
        settings: &Settings,
        messaging: MessagingServiceSync,
    ) -> ClientPacketHandler {
        let identity_provider = Box::new(
            FileIdentityManager::new(settings.authentication.password_file.as_str()).unwrap(),
        );

        ClientPacketHandler {
            rx,
            ctrl_c_rx,
            // sessions: SessionService::default(),
            messaging,
            identity_provider,
        }
    }

    pub async fn run(mut self) {
        loop {
            select! {
                _ = self.ctrl_c_rx.recv() => {
                    trace!("Stopping manager");
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
        debug!("New client {:?} connected", &packet.client_id);

        if let Some(user_name) = packet.user_name {
            let password = packet.password.unwrap();
            if let Err(e) = self.identity_provider.authenticate(&user_name, &password) {
                info!("Client {} authentication error: {:?}", &user_name, &e);

                let conn_ack = ConnAckPacket::new(false, ConnAckReturnCode::NotAuthorized);
                sender
                    .send(ServerEvent::ControlPacket(ControlPacket::ConnAck(conn_ack)))
                    .await
                    .unwrap();
                sender.send(ServerEvent::Disconnect).await.unwrap();
                return;
            }
        };

        let session_present = {
            let mut messaging = self.messaging.lock();
            let maybe_session = messaging.session_get_mut(&packet.client_id);
            if let Some(session) = maybe_session {
                session.set_last_activity(Utc::now());
                true
            } else {
                let session = Session::new(
                    packet.client_id,
                    address.ip(),
                    !packet.clean_session,
                    sender.clone(),
                    packet.keep_alive_seconds,
                    Utc::now(),
                );
                messaging.session_insert(session);
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

        let mut messaging = self.messaging.lock();
        messaging.disconnect(&client_id);

        // debug!(
        //     "Active sessions count: {:?}",
        //     messaging.session_count()
        // );
    }

    async fn on_connection_lost(&mut self, client_id: ClientId) {
        info!("Client {:?} disconnected unexpectedly", &client_id);

        let mut messaging = self.messaging.lock();
        messaging.connection_lost(&client_id);

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

        let senders_to_publish = {
            let mut messaging = self.messaging.lock();
            messaging.senders_to_publish(&publish.message.topic)
        };

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
        debug!("Client subscribed to topics {:?}", &subscribe.subscriptions);

        let mut return_codes = Vec::new();

        {
            let mut messaging = self.messaging.lock();

            for subscription in subscribe.subscriptions {
                // each subscription request must be handled as a separate subscribe packet
                let return_code = messaging.subscribe(client_id, &subscription);
                return_codes.push(return_code);
            }
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
        debug!("Client unsubscribed from topics {:?}", &unsubscribe.topics);

        {
            let mut messaging = self.messaging.lock();
            messaging.unsubscribe(client_id, &unsubscribe.topics);
        }

        let unsub_ack = UnSubAckPacket::new(unsubscribe.packet_id);
        sender
            .send(ServerEvent::ControlPacket(UnsubAck(unsub_ack)))
            .await
            .unwrap();
    }

    async fn on_ping_req(&mut self, sender: Sender<ServerEvent>, client_id: &ClientId) {
        let session_present = {
            let mut messaging = self.messaging.lock();

            let maybe_session = messaging.session_get_mut(client_id);

            if let Some(session) = maybe_session {
                session.set_last_activity(Utc::now());
                true
            } else {
                false
            }
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
