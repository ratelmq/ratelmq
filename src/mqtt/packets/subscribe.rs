use crate::mqtt::packets::QoS;
use crate::mqtt::subscription::Subscription;
use crate::mqtt::transport::mqtt_bytes_stream::MqttBytesStream;
use crate::mqtt::transport::packet_decoder::PacketDecoder;
use async_trait::async_trait;
use log::trace;
use tokio::io::{Error, ErrorKind};

#[derive(Debug, PartialEq, Clone, Default)]
pub struct SubscribePacket {
    pub packet_id: u16,

    pub subscriptions: Vec<Subscription>,
}

#[async_trait]
impl PacketDecoder for SubscribePacket {
    fn parse_fixed_header_flags(&mut self, flags_byte: u8) -> Result<(), Error> {
        if flags_byte != 0b10000010 {
            return Err(tokio::io::Error::new(
                ErrorKind::InvalidData,
                "Malformed fixed header",
            ));
        }

        Ok(())
    }

    fn variable_header_size(&self) -> usize {
        2
    }

    async fn parse_variable_header(
        &mut self,
        buffer: &mut MqttBytesStream,
    ) -> Result<usize, Error> {
        trace!("Parsing variable header");

        self.packet_id = buffer.get_u16().await?;
        trace!("\tpacket id: {:#?}", &self.packet_id);

        Ok(self.variable_header_size())
    }

    async fn parse_payload(
        &mut self,
        buffer: &mut MqttBytesStream,
        mut remaining_length: u64,
    ) -> Result<usize, Error> {
        if remaining_length == 0 {
            return Err(tokio::io::Error::new(
                ErrorKind::InvalidData,
                "Malformed payload",
            ));
        }

        while remaining_length > 0 {
            let topic = buffer.get_string().await?;
            let qos = buffer.get_u8().await?;
            if qos > 2 {
                return Err(tokio::io::Error::new(
                    ErrorKind::InvalidData,
                    "Malformed QoS",
                ));
            }

            remaining_length -= (topic.len() as u64 + 3/* 2 topic length + QoS*/);

            let subscription = Subscription::new(topic, QoS::from_bits(qos));
            self.subscriptions.push(subscription);
        }

        Ok(remaining_length as usize)
    }
}
