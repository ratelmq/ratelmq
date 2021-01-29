use async_trait::async_trait;
use tokio::io::Error;

use crate::mqtt::packets::PACKET_TYPE_PUB_COMP;
use crate::mqtt::transport::mqtt_bytes_stream::MqttBytesStream;
use crate::mqtt::transport::packet_encoder::{encode_remaining_length, PacketEncoder};

#[derive(Debug, PartialEq, Clone, Default)]
pub struct PubCompPacket {
    pub packet_id: u16,
}

impl PubCompPacket {
    pub fn new(packet_id: u16) -> Self {
        PubCompPacket { packet_id }
    }
}

#[async_trait]
impl PacketEncoder for PubCompPacket {
    async fn encode_fixed_header(&self, buffer: &mut MqttBytesStream) -> Result<(), Error> {
        buffer.put_u8(PACKET_TYPE_PUB_COMP << 4).await?;

        let remaining_length: u64 = 2 /* packet id */;
        encode_remaining_length(remaining_length, buffer).await?;

        Ok(())
    }

    async fn encode_variable_header(&self, buffer: &mut MqttBytesStream) -> Result<(), Error> {
        buffer.put_u16(self.packet_id).await?;

        Ok(())
    }
}
