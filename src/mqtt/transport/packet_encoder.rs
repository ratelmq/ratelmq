use async_trait::async_trait;
use tokio::io::Error;

use crate::mqtt::transport::mqtt_bytes_stream::MqttBytesStream;

#[async_trait]
pub trait PacketEncoder {
    async fn encode_fixed_header(&self, buffer: &mut MqttBytesStream) -> Result<(), Error>;
    async fn encode_variable_header(&self, _buffer: &mut MqttBytesStream) -> Result<(), Error> {
        Ok(())
    }
    async fn encode_body(&self, _buffer: &mut MqttBytesStream) -> Result<(), Error> {
        Ok(())
    }
}

pub async fn encode_remaining_length(
    mut remaining_length: u64,
    buffer: &mut MqttBytesStream,
) -> Result<(), Error> {
    loop {
        let mut encoded_byte: u8 = (remaining_length % 128) as u8;
        remaining_length /= 128;

        if remaining_length > 0 {
            encoded_byte |= 128;
        }

        buffer.put_u8(encoded_byte).await?;

        if remaining_length == 0 {
            break;
        }
    }
    Ok(())
}
