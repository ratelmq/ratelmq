use async_trait::async_trait;

use crate::mqtt::transport::mqtt_bytes_stream::MqttBytesStream;
use crate::mqtt::transport::packet_decoder::PacketDecoder;
use tokio::io::Error;

#[derive(Debug, PartialEq, Clone, Default)]
pub struct DisconnectPacket {}

#[async_trait]
impl PacketDecoder for DisconnectPacket {
    fn parse_fixed_header_flags(&mut self, _: u8) -> Result<(), Error> {
        Ok(())
    }

    fn variable_header_size(&self) -> usize {
        0
    }

    async fn parse_variable_header(
        &mut self,
        _buffer: &mut MqttBytesStream,
    ) -> Result<usize, Error> {
        Ok(self.variable_header_size())
    }

    async fn parse_payload(
        &mut self,
        _buffer: &mut MqttBytesStream,
        _remaining_length: u64,
    ) -> Result<usize, Error> {
        Ok(0)
    }
}
