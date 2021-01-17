use tokio::net::TcpStream;

use crate::mqtt::packets::{ControlPacket};
use crate::mqtt::transport::mqtt_packets_stream::MqttPacketsStream;
use crate::mqtt::transport::packet_encoder::PacketEncoder;
use tokio::io::Error;

pub struct Connection {
    mqtt_stream: MqttPacketsStream,
}

impl Connection {
    pub fn new(stream: TcpStream, buffer_size: usize) -> Connection {
        Connection {
            mqtt_stream: MqttPacketsStream::new(buffer_size, stream),
        }
    }

    pub async fn read_packet(&mut self) -> Result<ControlPacket, Error> {
        println!("Reading packet");
        self.mqtt_stream.read_packet().await
    }

    pub async fn write_packet(
        &mut self,
        packet: &(impl PacketEncoder + Sync),
    ) -> Result<(), Error> {
        self.mqtt_stream.write_packet(packet).await?;
        Ok(())
    }
}
