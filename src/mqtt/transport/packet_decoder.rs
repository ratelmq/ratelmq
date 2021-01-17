use async_trait::async_trait;
use bytes::{Buf, BytesMut};
use tokio::io::Error;

use crate::mqtt::codec::MqttCodec;
use crate::mqtt::transport::mqtt_bytes_stream::MqttBytesStream;
use crate::mqtt::utils::{parse_string, parse_u16};

#[async_trait]
pub trait PacketDecoder {
    fn parse_fixed_header_flags(&self, flags: u8) -> Result<(), Error>;

    fn variable_header_size(&self) -> usize {
        0
    }

    async fn parse_variable_header(
        &mut self,
        buffer: &mut MqttBytesStream,
    ) -> Result<usize, Error> {
        Ok(0)
    }

    async fn parse_payload(&mut self, buffer: &mut MqttBytesStream) -> Result<usize, Error> {
        Ok(0)
    }
}
