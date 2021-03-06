use crate::mqtt::events::{ClientEvent, ServerEvent};
use crate::mqtt::packets::ControlPacket;
use crate::mqtt::transport::mqtt_bytes_stream::{MqttBytesReadStream, MqttBytesWriteStream};
use crate::mqtt::transport::packet_decoder::read_packet;
use crate::mqtt::transport::packet_encoder::write_packet;
use log::{debug, error, info, trace, warn};
use std::net::SocketAddr;
use tokio::io::Error;
use tokio::net::{TcpListener, TcpStream};
use tokio::select;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::{broadcast, mpsc};

pub struct MqttListener {
    listener: TcpListener,
    client_event_tx: mpsc::Sender<ClientEvent>,
    ctrl_c_rx: broadcast::Receiver<()>,
}

impl MqttListener {
    pub async fn bind(
        address: &str,
        client_event_tx: mpsc::Sender<ClientEvent>,
        ctrl_c_rx: broadcast::Receiver<()>,
    ) -> Result<MqttListener, Error> {
        debug!("Binding MQTT TCP to {}", &address);

        let listener = TcpListener::bind(address).await.unwrap();
        info!("Listening for MQTT TCP connections on {}", &address);

        let mqtt_listener = MqttListener {
            listener,
            client_event_tx,
            ctrl_c_rx,
        };
        Ok(mqtt_listener)
    }

    pub async fn start_accepting(mut self) {
        loop {
            select! {
                _ = self.ctrl_c_rx.recv() => {
                    trace!("Stopping listener");
                    break;
                }
                _ = Self::accept(&self.listener, &self.client_event_tx) => {}
            }
        }
    }

    async fn accept(listener: &TcpListener, client_event_tx: &mpsc::Sender<ClientEvent>) {
        match listener.accept().await {
            Ok((socket, address)) => {
                let client_event_tx = client_event_tx.clone();
                tokio::spawn(async move {
                    Self::handle_connection(socket, client_event_tx, address).await;
                });
            }
            Err(e) => {
                error!("Error while accepting connection: {:?}", e);
            }
        }
    }

    async fn handle_connection(
        socket: TcpStream,
        client_event_tx: Sender<ClientEvent>,
        address: SocketAddr,
    ) {
        let (tcp_read, tcp_write) = socket.into_split();
        let (server_event_tx, server_event_rx) = mpsc::channel(32);

        let mut write_stream = MqttBytesWriteStream::new(4096, tcp_write);

        tokio::spawn(async move {
            Self::connection_write_loop(server_event_rx, &mut write_stream).await;
        });

        let mut read_stream = MqttBytesReadStream::new(4096, tcp_read);

        tokio::spawn(async move {
            Self::connection_read_loop(client_event_tx, server_event_tx, &mut read_stream, address)
                .await;
        });
    }

    async fn connection_read_loop(
        client_event_tx: Sender<ClientEvent>,
        server_event_tx: Sender<ServerEvent>,
        mut read_stream: &mut MqttBytesReadStream,
        address: SocketAddr,
    ) {
        // the first packet must be CONNECT - MQTT-3.1.0-1
        let client_id;
        if let Ok(packet) = read_packet(&mut read_stream).await {
            trace!("Read the first packet: {:?}", &packet);

            if let ControlPacket::Connect(c) = packet {
                // todo: handle empty client id
                client_id = c.client_id.clone();
                let event = ClientEvent::Connected(c, address, server_event_tx.clone());
                if let Err(e) = client_event_tx.send(event).await {
                    error!("Error while sending client event to be processed: {}", &e);
                }
            } else {
                warn!("The first received packet is not CONNECT");
                return;
            }
        } else {
            return;
        }

        while let Ok(packet) = read_packet(&mut read_stream).await {
            trace!("Read packet: {:?}", &packet);

            let event =
                ClientEvent::ControlPacket(client_id.clone(), packet, server_event_tx.clone());
            if let Err(e) = client_event_tx.send(event).await {
                error!("Error while sending client event to be processed: {}", &e);
            }
        }
        trace!("Client read task ended");
    }

    async fn connection_write_loop(
        mut server_event_rx: Receiver<ServerEvent>,
        mut write_stream: &mut MqttBytesWriteStream,
    ) {
        while let Some(event) = server_event_rx.recv().await {
            trace!("Received server event: {:?}", &event);

            match event {
                ServerEvent::ControlPacket(packet) => {
                    trace!("Writing packet: {:?}", &packet);
                    if let Err(e) = write_packet(&mut write_stream, packet).await {
                        error!("Error while writing packet: {:?}", &e);
                    }
                }
                ServerEvent::Disconnect => {
                    break;
                }
            }
        }

        trace!("Client write task ended");
    }
}
