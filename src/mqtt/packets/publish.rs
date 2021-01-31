use crate::mqtt::packets::{QoS, PACKET_TYPE_PUBLISH};
use crate::mqtt::transport::mqtt_bytes_stream::MqttBytesStream;
use crate::mqtt::transport::packet_decoder::PacketDecoder;
use crate::mqtt::transport::packet_encoder::{encode_remaining_length, PacketEncoder};
use async_trait::async_trait;
use bitflags::bitflags;
use bytes::BytesMut;
use log::trace;
use tokio::io::Error;

#[derive(Debug, PartialEq, Clone, Default)]
pub struct PublishPacket {
    pub dup: bool,
    pub qos: QoS,
    pub retain: bool,
    pub packet_id: Option<u16>,

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

        self.dup = flags.contains(FixedHeaderFlags::DUP);
        self.retain = flags.contains(FixedHeaderFlags::RETAIN);
        self.qos = QoS::from_bits((flags & FixedHeaderFlags::QOS).bits >> 1);
        Ok(())
    }

    fn variable_header_size(&self) -> usize {
        let mut size = self.topic.len() + 2 /* len */;
        if self.qos > QoS::AtMostOnce {
            size += 2; // packet id
        };

        size
    }

    async fn parse_variable_header(
        &mut self,
        buffer: &mut MqttBytesStream,
    ) -> Result<usize, Error> {
        trace!("Parsing variable header");

        self.topic = buffer.get_string().await?;

        if self.qos > QoS::AtMostOnce {
            self.packet_id = Some(buffer.get_u16().await?);
        }

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

#[async_trait]
impl PacketEncoder for PublishPacket {
    async fn encode_fixed_header(&self, buffer: &mut MqttBytesStream) -> Result<(), Error> {
        let mut first_byte = PACKET_TYPE_PUBLISH << 4;
        if self.dup {
            first_byte |= FixedHeaderFlags::DUP.bits;
        }
        first_byte |= (self.qos as u8) << 1;
        if self.retain {
            first_byte |= FixedHeaderFlags::RETAIN.bits;
        }

        buffer.put_u8(first_byte).await?;

        let mut remaining_length: u64 = (self.topic.len() + 2) as u64;

        if self.qos > QoS::AtMostOnce {
            remaining_length += 2 /* packet id */;
        }

        remaining_length += self.body.len() as u64;

        encode_remaining_length(remaining_length, buffer).await?;
        Ok(())
    }

    async fn encode_variable_header(&self, buffer: &mut MqttBytesStream) -> Result<(), Error> {
        buffer.put_string(self.topic.as_str()).await?;

        if let Some(packet_id) = self.packet_id {
            buffer.put_u16(packet_id).await?;
        }

        Ok(())
    }

    async fn encode_body(&self, buffer: &mut MqttBytesStream) -> Result<(), Error> {
        // todo: refactor to eliminated clone
        buffer.put_bytes(self.body.clone()).await?;
        Ok(())
    }
}
