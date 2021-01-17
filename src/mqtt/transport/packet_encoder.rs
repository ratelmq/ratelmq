use bytes::{BufMut, BytesMut};
use tokio::io::Error;
use async_trait::async_trait;

use crate::mqtt::transport::mqtt_bytes_stream::MqttBytesStream;

#[async_trait]
pub trait PacketEncoder {
    async fn encode_fixed_header(&self, buffer: &mut MqttBytesStream) -> Result<(), Error>;
    async fn encode_variable_header(&self, buffer: &mut MqttBytesStream) -> Result<(), Error>;
    async fn encode_body(&self, buffer: &mut MqttBytesStream) -> Result<(), Error> {
        Ok(())
    }
}

pub async fn encode_remaining_length(mut remaining_length: u64, buffer: &mut MqttBytesStream) -> Result<(), Error> {
    while remaining_length > 0 {
        let mut encoded_byte: u8 = (remaining_length % 128) as u8;
        remaining_length = remaining_length / 128;

        if remaining_length > 0 {
            encoded_byte = encoded_byte | 128;
        }

        buffer.put_u8(encoded_byte).await?;
    }
    Ok(())
}

