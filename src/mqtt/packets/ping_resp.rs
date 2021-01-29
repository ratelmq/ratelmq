use async_trait::async_trait;

use crate::mqtt::packets::PACKET_TYPE_PING_RESP;
use crate::mqtt::transport::mqtt_bytes_stream::MqttBytesStream;
use crate::mqtt::transport::packet_encoder::{encode_remaining_length, PacketEncoder};
use tokio::io::Error;

#[derive(Debug, PartialEq, Clone, Default)]
pub struct PingRespPacket {}

#[async_trait]
impl PacketEncoder for PingRespPacket {
    async fn encode_fixed_header(&self, buffer: &mut MqttBytesStream) -> Result<(), Error> {
        buffer.put_u8(PACKET_TYPE_PING_RESP << 4).await?;

        const REMAINING_LENGTH: u64 = 0;
        encode_remaining_length(REMAINING_LENGTH, buffer).await?;

        Ok(())
    }
}
