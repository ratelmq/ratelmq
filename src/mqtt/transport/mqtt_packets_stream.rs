use crate::mqtt::packets::ControlPacket;
use crate::mqtt::packets::ControlPacket::{Connect, Publish};
use crate::mqtt::transport::mqtt_bytes_stream::MqttBytesStream;
use crate::mqtt::transport::packet_decoder::{decode_remaining_length, PacketDecoder};
use crate::mqtt::transport::packet_encoder::PacketEncoder;
use log::trace;
use tokio::io::Error;
use tokio::net::TcpStream;

pub struct MqttPacketsStream {
    mqtt_stream: MqttBytesStream,
}

impl MqttPacketsStream {
    pub fn new(buffer_size: usize, stream: TcpStream) -> MqttPacketsStream {
        MqttPacketsStream {
            // stream
            mqtt_stream: MqttBytesStream::new(8096, 8096, stream),
        }
    }

    pub async fn read_packet(&mut self) -> Result<ControlPacket, Error> {
        let first_byte = self.mqtt_stream.get_u8().await?;
        let packet_type = first_byte >> 4;
        let mut remaining_length = decode_remaining_length(&mut self.mqtt_stream).await?;

        let mut control_packet = ControlPacket::new(packet_type);

        match &mut control_packet {
            ControlPacket::Connect(cp) => self.parse(cp, first_byte, remaining_length).await?,
            ControlPacket::Publish(cp) => self.parse(cp, first_byte, remaining_length).await?,
            ControlPacket::Disconnect(dp) => self.parse(dp, first_byte, remaining_length).await?,
            _ => unimplemented!(),
        }

        Ok(control_packet)
    }

    async fn parse(
        &mut self,
        cp: &mut (impl PacketDecoder + Send),
        first_byte: u8,
        mut remaining_length: u64,
    ) -> Result<(), Error> {
        cp.parse_fixed_header_flags(first_byte)?;
        remaining_length -= cp.parse_variable_header(&mut self.mqtt_stream).await? as u64;
        cp.parse_payload(&mut self.mqtt_stream, remaining_length)
            .await?;
        Ok(())
    }

    pub async fn write_packet(
        &mut self,
        packet: &(impl PacketEncoder + Sync),
    ) -> Result<(), Error> {
        trace!("Writing packet");
        packet.encode_fixed_header(&mut self.mqtt_stream).await?;
        packet.encode_variable_header(&mut self.mqtt_stream).await?;
        packet.encode_body(&mut self.mqtt_stream).await?;
        self.mqtt_stream.finish_packet().await?;
        Ok(())
    }
}
