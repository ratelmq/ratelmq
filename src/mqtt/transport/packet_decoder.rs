use async_trait::async_trait;
use tokio::io::{Error, ErrorKind};

use crate::mqtt::transport::mqtt_bytes_stream::MqttBytesStream;

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

    async fn parse_payload(
        &mut self,
        buffer: &mut MqttBytesStream,
        remaining_length: u64,
    ) -> Result<usize, Error> {
        Ok(0)
    }
}

pub async fn decode_remaining_length(buffer: &mut MqttBytesStream) -> Result<u64, Error> {
    let mut remaining_length = 0u64;
    let mut multiplier = 1u64;

    loop {
        let byte = buffer.get_u8().await?;
        remaining_length += (byte & 127) as u64 * multiplier;
        multiplier *= 128;

        if multiplier > 128 * 128 * 128 {
            return Err(tokio::io::Error::new(
                ErrorKind::InvalidData,
                "Malformed remaining length multiplier",
            ));
        }

        let continuation_bit = byte & 128;
        if continuation_bit == 0 {
            break;
        }
    }

    Ok(remaining_length)
}
