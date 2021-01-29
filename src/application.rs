use std::sync::{Arc, Mutex};

use log::{debug, error, info, trace};
use tokio::net::{TcpListener, TcpStream};

use crate::broker::manager::Manager;
use crate::mqtt::connection::Connection;
use crate::mqtt::listener::MqttListener;
use crate::mqtt::packets::ping_resp::PingRespPacket;
use crate::mqtt::packets::suback::{SubAckPacket, SubAckReturnCode};
use crate::mqtt::packets::unsuback::UnSubAckPacket;
use crate::mqtt::packets::ControlPacket;

type MqttManager = Arc<Mutex<Manager>>;

pub struct Application {
    manager: MqttManager,
}

impl Application {
    pub fn new() -> Application {
        Application {
            manager: Arc::new(Mutex::new(Manager::new())),
        }
    }

    pub async fn run(&self) {
        info!("Initializing RatelMQ...");

        let _l = MqttListener::new("127.0.0.1:1883");

        let listener = TcpListener::bind("127.0.0.1:1883").await.unwrap();

        info!("Initialized RatelMQ");

        loop {
            let (socket, _) = listener.accept().await.unwrap();

            let manager = self.manager.clone();
            tokio::spawn(async move {
                process(socket, manager).await;
            });
        }
    }
}

async fn process(socket: TcpStream, manager: MqttManager) {
    let mut connection = Connection::new(socket, 4096);
    loop {
        match &connection.read_packet().await {
            Ok(packet) => {
                trace!("Received complete packet ");
                match packet {
                    ControlPacket::Connect(cp) => {
                        debug!("Received CONNECT packet");

                        let conn_ack = {
                            let manager = manager.lock().unwrap();
                            manager.connect(&cp)
                        };

                        connection.write_packet(&conn_ack).await.unwrap();
                    }
                    ControlPacket::Publish(pp) => {
                        debug!(
                            "Received PUBLISH packet: topic={} payload={:?}",
                            &pp.topic, &pp.body
                        );

                        if let Some(_publish_result) = {
                            let manager = manager.lock().unwrap();
                            manager.publish(&pp)
                        } {
                            debug!("Response");
                        };
                    }
                    ControlPacket::Subscribe(sp) => {
                        debug!(
                            "Received SUBSCRIBE packet: packet_id={} subscriptions={:?}",
                            &sp.packet_id, &sp.subscriptions
                        );

                        let sub_ack = SubAckPacket::new(
                            sp.packet_id,
                            sp.subscriptions
                                .iter()
                                .map(|_sub| SubAckReturnCode::Failure)
                                .collect(),
                        );
                        connection.write_packet(&sub_ack).await.unwrap();
                    }
                    ControlPacket::Unsubscribe(up) => {
                        debug!(
                            "Received UNSUBSCRIBE packet: packet_id={} topics={:?}",
                            &up.packet_id, &up.topics
                        );

                        let unsub_ack = UnSubAckPacket::new(up.packet_id);
                        connection.write_packet(&unsub_ack).await.unwrap();
                    }
                    ControlPacket::PingReq(_) => {
                        debug!("Received PINGREQ packet");
                        let resp = PingRespPacket::default();
                        connection.write_packet(&resp).await.unwrap();
                    }
                    ControlPacket::Disconnect(dp) => {
                        debug!("Received DISCONNECT packet");
                        {
                            let manager = manager.lock().unwrap();
                            manager.disconnect(&dp)
                        };
                    }
                    _ => unimplemented!(),
                }
                // } else {
                //     println!("Client closed connection");
                //     break;
                // }
            }
            Err(error) => {
                error!("Received error: {:#?}", error);
                break;
            }
        }
    }
}
