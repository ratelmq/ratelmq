use crate::mqtt::packets::{ProtocolVersion, QoS};
use crate::mqtt::transport::mqtt_bytes_stream::MqttBytesStream;
use crate::mqtt::transport::packet_decoder::PacketDecoder;
use async_trait::async_trait;
use bytes::BytesMut;
use log::{debug, trace};
use tokio::io::Error;

#[derive(Debug, PartialEq, Clone, Default)]
pub struct PublishPacket {
    pub qos: QoS,
    pub retain: bool,

    pub topic: String,
    pub body: BytesMut,
}

#[async_trait]
impl PacketDecoder for PublishPacket {
    fn parse_fixed_header_flags(&self, _: u8) -> Result<(), Error> {
        Ok(())
    }

    fn variable_header_size(&self) -> usize {
        self.topic.len() + 2
    }

    async fn parse_variable_header(
        &mut self,
        buffer: &mut MqttBytesStream,
    ) -> Result<usize, Error> {
        trace!("Parsing variable header");

        self.topic = buffer.get_string().await?;
        // todo: packet identifier

        trace!("\ttopic: {:#?}", &self.topic);

        Ok(self.variable_header_size())
    }

    async fn parse_payload(
        &mut self,
        buffer: &mut MqttBytesStream,
        remaining_length: u64,
    ) -> Result<usize, Error> {
        self.body = buffer.get_bytes(remaining_length as usize).await?;

        Ok(remaining_length as usize)
    }
}
