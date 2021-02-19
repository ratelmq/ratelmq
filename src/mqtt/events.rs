use crate::mqtt::packets::{ClientId, ConnectPacket, ControlPacket};
use std::net::SocketAddr;
use tokio::sync::mpsc::Sender;

#[derive(Debug)]
pub enum ClientEvent {
    Connected(ConnectPacket, SocketAddr, Sender<ServerEvent>),
    ControlPacket(ClientId, ControlPacket, Sender<ServerEvent>),
    Disconnected(ClientId),
    ConnectionLost(ClientId),
}

#[derive(Debug)]
pub enum ServerEvent {
    ControlPacket(ControlPacket),
    Disconnect,
}
