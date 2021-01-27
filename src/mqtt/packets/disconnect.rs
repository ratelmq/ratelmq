use async_trait::async_trait;
use tokio::io::Error;

use crate::mqtt::transport::packet_decoder::PacketDecoder;

#[derive(Debug, PartialEq, Clone, Default)]
pub struct DisconnectPacket {}

#[async_trait]
impl PacketDecoder for DisconnectPacket {
    fn parse_fixed_header_flags(&mut self, _: u8) -> Result<(), Error> {
        Ok(())
    }
}
