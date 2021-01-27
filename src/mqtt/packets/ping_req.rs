use async_trait::async_trait;

use crate::mqtt::transport::packet_decoder::PacketDecoder;
use tokio::io::{Error, ErrorKind};

#[derive(Debug, PartialEq, Clone, Default)]
pub struct PingReqPacket {}

#[async_trait]
impl PacketDecoder for PingReqPacket {
    fn parse_fixed_header_flags(&mut self, flags: u8) -> Result<(), Error> {
        if flags != 0b11000000 {
            return Err(tokio::io::Error::new(
                ErrorKind::InvalidData,
                "Malformed fixed header",
            ));
        }

        Ok(())
    }
}
