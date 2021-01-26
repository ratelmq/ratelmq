use crate::mqtt::packets::QoS;
use crate::mqtt::transport::mqtt_bytes_stream::MqttBytesStream;
use crate::mqtt::transport::packet_decoder::PacketDecoder;
use async_trait::async_trait;
use bitflags::bitflags;
use bytes::BytesMut;
use log::trace;
use tokio::io::Error;

#[derive(Debug, PartialEq, Clone, Default)]
pub struct PublishPacket {
    pub qos: QoS,
    pub retain: bool,

    pub topic: String,
    pub body: BytesMut,
}

bitflags! {
    struct FixedHeaderFlags: u8 {
        const RETAIN =  0b00000001;
        const QOS_1 =   0b00000010;
        const QOS_2 =   0b00000100;
        const DUP =     0b00001000;
        const QOS = Self::QOS_1.bits | Self::QOS_2.bits;
    }
}

#[async_trait]
impl PacketDecoder for PublishPacket {
    fn parse_fixed_header_flags(&mut self, flags_byte: u8) -> Result<(), Error> {
        let flags = FixedHeaderFlags::from_bits_truncate(flags_byte);

        self.retain = flags.contains(FixedHeaderFlags::RETAIN);
        self.qos = QoS::from_bits((flags & FixedHeaderFlags::QOS).bits >> 1);
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
