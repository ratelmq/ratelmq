use crate::broker::session::{InMemorySessionRepository, Session, SessionService};
use crate::mqtt::events::{ClientEvent, ServerEvent};
use crate::mqtt::packets::connack::ConnAckReturnCode;
use crate::mqtt::packets::suback::{SubAckPacket, SubAckReturnCode};
use crate::mqtt::packets::subscribe::SubscribePacket;
use crate::mqtt::packets::unsuback::UnSubAckPacket;
use crate::mqtt::packets::unsubscribe::UnsubscribePacket;
use crate::mqtt::packets::ControlPacket::{ConnAck, PingResp, Publish, SubAck, UnsubAck};
use crate::mqtt::packets::*;
use log::{debug, error, info, warn, trace};
use std::collections::HashMap;
use std::net::SocketAddr;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::{broadcast, mpsc};
use tokio::select;

type Subs = HashMap<String, Vec<ClientId>>;

pub struct Manager {
    rx: mpsc::Receiver<ClientEvent>,
    ctrl_c_rx: broadcast::Receiver<()>,
    sessions: SessionService<InMemorySessionRepository>,
    subscriptions: Subs,
}

impl Manager {
    pub fn new(rx: Receiver<ClientEvent>, ctrl_c_rx: broadcast::Receiver<()>) -> Manager {
        Manager {
            rx,
            ctrl_c_rx,
            sessions: SessionService::default(),
            subscriptions: Subs::new(),
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

    async fn on_packet(&mut self, client_id: String, packet: ControlPacket, tx: Sender<ServerEvent>) {
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
            ControlPacket::Unsubscribe(p) => {
                self.on_unsubscribe(tx, p, &client_id).await
            }
            // ControlPacket::UnsubAck(_) => {}
            ControlPacket::PingReq => self.on_ping_req(tx).await,
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

        let session_present = self.sessions.exists(&packet.client_id);
        if !session_present {
            let session = Session::new(
                packet.client_id,
                address.ip(),
                !packet.clean_session,
                sender.clone(),
            );
            self.sessions.insert(session);
        }

        let conn_ack = ConnAckPacket::new(session_present, ConnAckReturnCode::Accepted);

        sender
            .send(ServerEvent::ControlPacket(ConnAck(conn_ack)))
            .await
            .unwrap();

        debug!("Active sessions count: {:?}", self.sessions.count());
    }

    async fn on_disconnect(&mut self, client_id: ClientId) {
        debug!("Client {:?} disconnected", &client_id);
        self.sessions.delete(&client_id);

        debug!("Active sessions count: {:?}", self.sessions.count());
    }

    async fn on_connection_lost(&mut self, client_id: ClientId) {
        info!("Client {:?} disconnected unexpectedly", &client_id);
        self.sessions.delete(&client_id);

        debug!("Active sessions count: {:?}", self.sessions.count());
    }

    async fn on_publish(
        &self,
        _sender: Sender<ServerEvent>,
        publish: PublishPacket,
        client_id: ClientId,
    ) {
        debug!(
            "Client {:?} published message on topic {:?}",
            &client_id, &publish.topic
        );

        if let Some(client_ids) = self.subscriptions.get(&publish.topic) {
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

    async fn on_subscribe(
        &mut self,
        sender: Sender<ServerEvent>,
        subscribe: SubscribePacket,
        client_id: &ClientId,
    ) {
        debug!("Client subscribed to topics {:?}", &subscribe.subscriptions);

        let mut return_codes = Vec::new();
        for subscription in subscribe.subscriptions {
            // each subscription request must be handled as a separate subscribe packet
            self.subscriptions
                .entry(subscription.topic().to_string())
                .or_insert(Vec::new())
                .push(client_id.clone());
            return_codes.push(SubAckReturnCode::Failure);
        }

        let sub_ack = SubAckPacket::new(subscribe.packet_id, return_codes);
        sender
            .send(ServerEvent::ControlPacket(SubAck(sub_ack)))
            .await
            .unwrap();
    }

    async fn on_unsubscribe(
        &self,
        sender: Sender<ServerEvent>,
        unsubscribe: UnsubscribePacket,
        _client_id: &ClientId,
    ) {
        debug!("Client unsubscribed from topics {:?}", &unsubscribe.topics);

        let unsub_ack = UnSubAckPacket::new(unsubscribe.packet_id);
        sender
            .send(ServerEvent::ControlPacket(UnsubAck(unsub_ack)))
            .await
            .unwrap();
    }

    async fn on_ping_req(&self, sender: Sender<ServerEvent>) {
        sender
            .send(ServerEvent::ControlPacket(PingResp))
            .await
            .unwrap();
    }
}
