use crate::mqtt::action::Action;
use crate::mqtt::packets::connack::ConnAckReturnCode;
use crate::mqtt::packets::suback::{SubAckPacket, SubAckReturnCode};
use crate::mqtt::packets::subscribe::SubscribePacket;
use crate::mqtt::packets::unsuback::UnSubAckPacket;
use crate::mqtt::packets::unsubscribe::UnsubscribePacket;
use crate::mqtt::packets::ControlPacket::{ConnAck, PingResp, Publish, SubAck, UnsubAck};
use crate::mqtt::packets::*;
use log::{debug, error, info};
use std::collections::HashMap;
use tokio::sync::mpsc::{Receiver, Sender};

pub struct Manager {
    rx: Receiver<Action>,
}

type Subs = HashMap<String, Vec<Sender<ControlPacket>>>;

impl Manager {
    pub fn new(rx: Receiver<Action>) -> Manager {
        Manager { rx }
    }

    pub async fn run(mut self) {
        let mut subs: Subs = HashMap::new();

        while let Some(action) = self.rx.recv().await {
            let packet = action.packet;
            log::trace!("Got packet {:?}", packet);

            match packet {
                ControlPacket::Connect(c) => self.on_connect(action.response, c).await,
                // ControlPacket::ConnAck(_) => {}
                ControlPacket::Publish(p) => self.on_publish(action.response, p, &subs).await,
                // ControlPacket::PubAck(_) => {}
                // ControlPacket::PubRec(_) => {}
                // ControlPacket::PubRel(_) => {}
                // ControlPacket::PubComp(_) => {}
                ControlPacket::Subscribe(p) => {
                    self.on_subscribe(action.response, p, &mut subs).await
                }
                // ControlPacket::SubAck(_) => {}
                ControlPacket::Unsubscribe(p) => {
                    self.on_unsubscribe(action.response, p, &subs).await
                }
                // ControlPacket::UnsubAck(_) => {}
                ControlPacket::PingReq => self.on_ping_req(action.response).await,
                // ControlPacket::PingResp() => {}
                ControlPacket::Disconnect(p) => self.on_disconnect(p).await,
                _ => error!("Packet {} not supported", &packet),
            };
        }
    }

    async fn on_connect(&self, sender: Sender<ControlPacket>, connect_packet: ConnectPacket) {
        info!("New client {:?} connected", connect_packet.client_id);

        let conn_ack = ConnAckPacket {
            session_present: false,
            return_code: ConnAckReturnCode::Accepted,
        };

        sender.send(ConnAck(conn_ack)).await.unwrap();
    }

    async fn on_disconnect(&self, _disconnect_packet: DisconnectPacket) {
        debug!("Client disconnected");
    }

    async fn on_publish(
        &self,
        _sender: Sender<ControlPacket>,
        publish: PublishPacket,
        subs: &Subs,
    ) {
        debug!("Client published message on topic {}", &publish.topic);

        if let Some(senders) = subs.get(&publish.topic) {
            for s in senders {
                s.send(Publish(publish.clone())).await.unwrap();
            }
        }
    }

    async fn on_subscribe(
        &self,
        sender: Sender<ControlPacket>,
        subscribe: SubscribePacket,
        subs: &mut Subs,
    ) {
        debug!("Client subscribed to topics {:?}", &subscribe.subscriptions);

        let mut return_codes = Vec::new();
        for subscription in subscribe.subscriptions {
            // each subscription request must be handled as a separate subscribe packet
            subs.entry(subscription.topic().to_string())
                .or_insert(Vec::new())
                .push(sender.clone());
            return_codes.push(SubAckReturnCode::Failure);
        }

        let sub_ack = SubAckPacket::new(subscribe.packet_id, return_codes);
        sender.send(SubAck(sub_ack)).await.unwrap();
    }

    async fn on_unsubscribe(
        &self,
        sender: Sender<ControlPacket>,
        unsubscribe: UnsubscribePacket,
        _subs: &Subs,
    ) {
        debug!("Client unsubscribed from topics {:?}", &unsubscribe.topics);

        let unsub_ack = UnSubAckPacket::new(unsubscribe.packet_id);
        sender.send(UnsubAck(unsub_ack)).await.unwrap();
    }

    async fn on_ping_req(&self, sender: Sender<ControlPacket>) {
        sender.send(PingResp).await.unwrap();
    }
}
