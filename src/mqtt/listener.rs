use crate::mqtt::action::Action;
use crate::mqtt::packets::ControlPacket;
use crate::mqtt::transport::mqtt_bytes_stream::{MqttBytesReadStream, MqttBytesWriteStream};
use crate::mqtt::transport::packet_decoder::read_packet;
use crate::mqtt::transport::packet_encoder::write_packet;
use log::{debug, error, trace};
use tokio::io::Error;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender};

pub struct MqttListener {
    listener: TcpListener,
    sender: Sender<Action>,
}

impl MqttListener {
    pub async fn bind(address: &str, sender: Sender<Action>) -> Result<MqttListener, Error> {
        debug!("Binding to {}", &address);

        let listener = TcpListener::bind(address).await.unwrap();
        let mqtt_listener = MqttListener { listener, sender };
        Ok(mqtt_listener)
    }

    pub async fn start_accepting(self) {
        loop {
            let (socket, _) = self.listener.accept().await.unwrap();

            let sender = self.sender.clone();
            tokio::spawn(async move {
                Self::handle_connection(socket, sender).await;
            });
        }
    }

    async fn handle_connection(socket: TcpStream, sender: Sender<Action>) {
        let (tcp_read, tcp_write) = socket.into_split();
        let (tx, rx) = mpsc::channel(32);

        let mut write_stream = MqttBytesWriteStream::new(4096, tcp_write);

        tokio::spawn(async move {
            Self::connection_write_loop(rx, &mut write_stream).await;
        });

        let mut read_stream = MqttBytesReadStream::new(4096, tcp_read);

        tokio::spawn(async move {
            Self::connection_read_loop(sender, tx, &mut read_stream).await;
        });
    }

    async fn connection_read_loop(
        sender: Sender<Action>,
        tx: Sender<ControlPacket>,
        mut read_stream: &mut MqttBytesReadStream,
    ) {
        while let Ok(packet) = read_packet(&mut read_stream).await {
            trace!("Read packet: {:?}", &packet);

            let action = Action::new(packet, tx.clone());
            if let Err(e) = sender.send(action).await {
                error!("Error while sending action to be processed: {}", &e);
            }
        }
        trace!("Client read task ended");
    }

    async fn connection_write_loop(
        mut rx: Receiver<ControlPacket>,
        mut write_stream: &mut MqttBytesWriteStream,
    ) {
        while let Some(response) = rx.recv().await {
            trace!("Writing packet: {:?}", &response);

            if let Err(e) = write_packet(&mut write_stream, response).await {
                error!("Error while writing packet: {:?}", &e);
            }
        }

        trace!("Client write task ended");
    }
}
