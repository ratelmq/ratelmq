use async_trait::async_trait;
use tokio::io::Error;

use crate::mqtt::packets::suback::SubAckReturnCode::Failure;
use crate::mqtt::transport::mqtt_bytes_stream::MqttBytesStream;
use crate::mqtt::transport::packet_encoder::{encode_remaining_length, PacketEncoder};

#[derive(Debug, PartialEq, Clone)]
pub enum SubAckReturnCode {
    SuccessQoS0 = 0x00,
    SuccessQoS1 = 0x01,
    SuccessQoS2 = 0x02,
    Failure = 0x80,
}

impl Default for SubAckReturnCode {
    fn default() -> Self {
        Failure
    }
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct SubAckPacket {
    pub packet_id: u16,

    pub return_codes: Vec<SubAckReturnCode>,
}

impl SubAckPacket {
    pub fn new(packet_id: u16, return_codes: Vec<SubAckReturnCode>) -> Self {
        SubAckPacket {
            packet_id,
            return_codes,
        }
    }
}

#[async_trait]
impl PacketEncoder for SubAckPacket {
    async fn encode_fixed_header(&self, buffer: &mut MqttBytesStream) -> Result<(), Error> {
        const PACKET_TYPE: u8 = 9;
        buffer.put_u8(PACKET_TYPE << 4).await?;

        let remaining_length: u64 = 2 /* packet id */ + self.return_codes.len() as u64;
        encode_remaining_length(remaining_length, buffer).await?;

        Ok(())
    }

    async fn encode_variable_header(&self, buffer: &mut MqttBytesStream) -> Result<(), Error> {
        buffer.put_u16(self.packet_id).await?;

        Ok(())
    }

    async fn encode_body(&self, buffer: &mut MqttBytesStream) -> Result<(), Error> {
        for return_code in &self.return_codes {
            buffer.put_u8(return_code.clone() as u8).await?;
        }
        Ok(())
    }
}
