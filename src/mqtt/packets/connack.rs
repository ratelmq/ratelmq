use async_trait::async_trait;
use tokio::io::Error;

use crate::mqtt::packets::PACKET_TYPE_CONN_ACK;
use crate::mqtt::transport::mqtt_bytes_stream::MqttBytesStream;
use crate::mqtt::transport::packet_encoder::{encode_remaining_length, PacketEncoder};

#[derive(Debug, PartialEq, Clone)]
pub enum ConnAckReturnCode {
    Accepted = 0x00,
    UnacceptableProtocolVersion,
    IdentifierRejected,
    ServerUnavailable,
    BadUserNameOrPassword,
    NotAuthorized,
}

impl Default for ConnAckReturnCode {
    fn default() -> Self {
        ConnAckReturnCode::Accepted
    }
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct ConnAckPacket {
    pub session_present: bool,
    pub return_code: ConnAckReturnCode,
}

#[async_trait]
impl PacketEncoder for ConnAckPacket {
    async fn encode_fixed_header(&self, buffer: &mut MqttBytesStream) -> Result<(), Error> {
        buffer.put_u8(PACKET_TYPE_CONN_ACK << 4).await?;

        const REMAINING_LENGTH: u64 = 2;
        encode_remaining_length(REMAINING_LENGTH, buffer).await?;

        Ok(())
    }

    async fn encode_variable_header(&self, buffer: &mut MqttBytesStream) -> Result<(), Error> {
        buffer.put_u8(self.session_present as u8).await?;
        buffer.put_u8(self.return_code.clone() as u8).await?;

        Ok(())
    }
}
