use crate::mqtt::transport::mqtt_bytes_stream::MqttBytesStream;
use tokio::net::TcpStream;
use crate::mqtt::packets::ControlPacket;
use tokio::io::Error;
use crate::mqtt::transport::packet_encoder::PacketEncoder;
use crate::mqtt::packets::ControlPacket::Connect;
use crate::mqtt::transport::packet_decoder::PacketDecoder;

pub struct MqttPacketsStream {
    mqtt_stream: MqttBytesStream,
}

impl MqttPacketsStream {
    pub fn new(buffer_size: usize, mut stream: TcpStream) -> MqttPacketsStream {
        MqttPacketsStream {
            // stream
            mqtt_stream: MqttBytesStream::new(8096, 8096, stream)
        }
    }

    pub async fn read_packet(&mut self) -> Result<ControlPacket, Error> {
        let first_byte = self.mqtt_stream.get_u8().await?;
        let packet_type = first_byte >> 4;
        let remaining_length = self.mqtt_stream.get_u8().await?;

        let mut control_packet = ControlPacket::new(packet_type);

        match &mut control_packet {
           Connect(cp) => {
               cp.parse_fixed_header_flags(first_byte);
               cp.parse_variable_header(&mut self.mqtt_stream).await?;
               cp.parse_payload(&mut self.mqtt_stream).await?;
           }
            _ => unimplemented!()
        }

        Ok(control_packet)
    }

    pub async fn write_packet(&mut self, packet: &(impl PacketEncoder + Sync)) -> Result<(), Error> {
        println!("Writing packet");
        packet.encode_fixed_header(&mut self.mqtt_stream).await?;
        packet.encode_variable_header(&mut self.mqtt_stream).await?;
        packet.encode_body(&mut self.mqtt_stream).await?;
        self.mqtt_stream.finish_packet().await?;
        Ok(())
    }
}
