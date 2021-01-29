use crate::mqtt::transport::mqtt_bytes_stream::MqttBytesStream;
use crate::mqtt::transport::packet_decoder::PacketDecoder;
use async_trait::async_trait;
use log::trace;
use tokio::io::{Error, ErrorKind};

#[derive(Debug, PartialEq, Clone, Default)]
pub struct UnsubscribePacket {
    pub packet_id: u16,

    pub topics: Vec<String>,
}

#[async_trait]
impl PacketDecoder for UnsubscribePacket {
    fn parse_fixed_header_flags(&mut self, flags_byte: u8) -> Result<(), Error> {
        if flags_byte != 0b10100010 {
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
        trace!("Parsing variable header");

        self.packet_id = buffer.get_u16().await?;
        trace!("\tpacket id: {:#?}", &self.packet_id);

        Ok(self.variable_header_size())
    }

    async fn parse_payload(
        &mut self,
        buffer: &mut MqttBytesStream,
        mut remaining_length: u64,
    ) -> Result<usize, Error> {
        if remaining_length == 0 {
            return Err(tokio::io::Error::new(
                ErrorKind::InvalidData,
                "Malformed payload",
            ));
        }

        while remaining_length > 0 {
            let topic = buffer.get_string().await?;

            remaining_length -= (topic.len() as u64 + 2/* 2 topic length */);

            self.topics.push(topic);
        }

        Ok(remaining_length as usize)
    }
}
