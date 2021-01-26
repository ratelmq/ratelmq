use async_trait::async_trait;
use tokio::io::Error;

use crate::mqtt::packets::{ProtocolVersion, QoS};
use crate::mqtt::transport::mqtt_bytes_stream::MqttBytesStream;
use crate::mqtt::transport::packet_decoder::PacketDecoder;
use log::trace;

use bitflags::bitflags;

#[derive(Debug, PartialEq, Clone, Default)]
pub struct ConnectPacket {
    // header
    pub version: ProtocolVersion,
    pub clean_session: bool,
    pub will_flag: bool,
    pub will_qos: QoS,
    pub will_retain: bool,
    pub keep_alive_seconds: u16,

    // payload
    pub client_id: String,
    pub will_topic: Option<String>,
    pub will_message: Option<String>,
    pub user_name: Option<String>,
    pub password: Option<String>,
}

bitflags! {
    struct ConnectFlags: u8 {
        const RESERVED =        0b00000001;
        const CLEAN_SESSION =   0b00000010;
        const WILL =            0b00000100;
        const WILL_QOS_1 =      0b00001000;
        const WILL_QOS_2 =      0b00010000;
        const WILL_RETAIN =     0b00100000;
        const PASSWORD =        0b01000000;
        const USERNAME =        0b10000000;
        const WILL_QOS = Self::WILL_QOS_1.bits | Self::WILL_QOS_2.bits;
    }
}

#[async_trait]
impl PacketDecoder for ConnectPacket {
    fn parse_fixed_header_flags(&mut self, _: u8) -> Result<(), Error> {
        Ok(())
    }

    fn variable_header_size(&self) -> usize {
        10
    }

    async fn parse_variable_header(
        &mut self,
        buffer: &mut MqttBytesStream,
    ) -> Result<usize, Error> {
        trace!("Parsing variable header");

        let protocol_name = buffer.get_string().await?;
        let protocol_level = buffer.get_u8().await?;
        let connect_flags_byte = buffer.get_u8().await?;
        let connect_flags = ConnectFlags::from_bits_truncate(connect_flags_byte);

        self.clean_session = connect_flags.contains(ConnectFlags::CLEAN_SESSION);

        self.keep_alive_seconds = buffer.get_u16().await?;

        trace!("Parsed variable header");
        trace!("\tprotocol name: {:#?}, protocol level: {:#08b}, connect flags: {:#08b}, keep alive: {}",
                 &protocol_name, &protocol_level, &connect_flags, &self.keep_alive_seconds);

        Ok(self.variable_header_size())
    }

    async fn parse_payload(
        &mut self,
        buffer: &mut MqttBytesStream,
        _remaining_length: u64,
    ) -> Result<usize, Error> {
        self.client_id = buffer.get_string().await?;

        trace!("\tclient id: {:#?}", &self.client_id);

        Ok(2 + self.client_id.len())
    }
}
