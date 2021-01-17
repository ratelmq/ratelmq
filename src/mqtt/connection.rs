use bytes::{Buf, BytesMut};
use tokio::io::{AsyncWriteExt, Error, ErrorKind};
use tokio::net::TcpStream;
use tokio::prelude::io::AsyncReadExt;

use crate::mqtt::codec::{Decoder, Encoder, MqttCodec};
use crate::mqtt::packets::{ControlPacket, ControlPacketType};
use crate::mqtt::parser::Parser;
use crate::mqtt::transport::mqtt_packets_stream::MqttPacketsStream;
use crate::mqtt::transport::packet_encoder::PacketEncoder;

pub struct Connection {
    mqtt_streamm: MqttPacketsStream,
}

impl Connection {
    pub fn new(stream: TcpStream, buffer_size: usize) -> Connection {
        Connection {
            mqtt_streamm: MqttPacketsStream::new(buffer_size, stream),
        }
    }

    pub async fn read_packet(&mut self) -> Result<ControlPacket, Error> {
        println!("Reading packet");
        self.mqtt_streamm.read_packet().await
    }

    pub async fn write_packet(&mut self, packet: &(impl PacketEncoder + Sync)) -> Result<(), Error> {
        self.mqtt_streamm.write_packet(packet).await?;
        Ok(())
    }
}
