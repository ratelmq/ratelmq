use std::sync::{Arc, Mutex};

use dotenv::dotenv;
use log::{debug, info, trace};
use tokio::net::{TcpListener, TcpStream};

use crate::broker::Manager;
use crate::mqtt::connection::Connection;
use crate::mqtt::listener::MqttListener;
use crate::mqtt::packets::ControlPacket;

type MqttManager = Arc<Mutex<Manager>>;

pub struct Application {
    manager: MqttManager,
}

impl Application {
    pub fn init() {
        dotenv().ok();
        env_logger::init();
    }

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
                eprintln!("Received error: {:#?}", error);
                break;
            }
        }
    }
}
