use async_trait::async_trait;
use tokio::io::{Error, ErrorKind};

use crate::mqtt::packets::PACKET_TYPE_PUB_REC;
use crate::mqtt::transport::mqtt_bytes_stream::MqttBytesStream;
use crate::mqtt::transport::packet_decoder::PacketDecoder;
use crate::mqtt::transport::packet_encoder::{encode_remaining_length, PacketEncoder};

#[derive(Debug, PartialEq, Clone, Default)]
pub struct PubRecPacket {
    pub packet_id: u16,
}

impl PubRecPacket {
    pub fn new(packet_id: u16) -> Self {
        PubRecPacket { packet_id }
    }
}

#[async_trait]
impl PacketDecoder for PubRecPacket {
    fn parse_fixed_header_flags(&mut self, flags: u8) -> Result<(), Error> {
        if flags != 0b01010000 {
            return Err(tokio::io::Error::new(
                ErrorKind::InvalidData,
                "Malformed fixed header",
            ));
        }

        Ok(())
    }

    fn variable_header_size(&self) -> usize {
        2
    }

    async fn parse_variable_header(
        &mut self,
        buffer: &mut MqttBytesStream,
    ) -> Result<usize, Error> {
        self.packet_id = buffer.get_u16().await?;

        Ok(self.variable_header_size())
    }
}

#[async_trait]
impl PacketEncoder for PubRecPacket {
    async fn encode_fixed_header(&self, buffer: &mut MqttBytesStream) -> Result<(), Error> {
        buffer.put_u8(PACKET_TYPE_PUB_REC << 4).await?;

        let remaining_length: u64 = 2 /* packet id */;
        encode_remaining_length(remaining_length, buffer).await?;

        Ok(())
    }

    async fn encode_variable_header(&self, buffer: &mut MqttBytesStream) -> Result<(), Error> {
        buffer.put_u16(self.packet_id).await?;

        Ok(())
    }
}
