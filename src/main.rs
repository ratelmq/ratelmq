use std::sync::{Arc, Mutex};

use dotenv::dotenv;
use log::info;
use tokio::net::{TcpListener, TcpStream};

use crate::broker::Manager;
use crate::mqtt::connection::Connection;
use crate::mqtt::packets::ControlPacket;

mod broker;
mod mqtt;

type MqttManager = Arc<Mutex<Manager>>;

#[tokio::main]
async fn main() {
    dotenv().ok();
    env_logger::init();

    info!("Initializing RatelMQ...");

    let manager = Arc::new(Mutex::new(Manager::new()));

    let listener = TcpListener::bind("127.0.0.1:1883").await.unwrap();

    info!("Initialized RatelMQ");

    loop {
        let (socket, _) = listener.accept().await.unwrap();

        let manager = manager.clone();
        tokio::spawn(async move {
            process(socket, manager).await;
        });
    }
}

async fn process(socket: TcpStream, manager: MqttManager) {
    let mut connection = Connection::new(socket, 4096);
    loop {
        match connection.read_packet().await {
            Ok(packet) => {
                println!("Received complete packet ");
                match packet {
                    ControlPacket::Connect(cp) => {
                        println!("Processing connect packet");

                        let conn_ack = {
                            let manager = manager.lock().unwrap();
                            manager.connect(&cp)
                        };

                        connection.write_packet(&conn_ack).await.unwrap();
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
