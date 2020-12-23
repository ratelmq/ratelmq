use crate::mqtt::packets::ControlPacketType;

trait Parse<T> {
    fn parse() -> T;
}

pub struct Parser {}

impl Parser {
    pub fn parse_packet_type(byte: u8) -> ControlPacketType {
        let packet_type = byte >> 4;
        unsafe { ControlPacketType::from(packet_type) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Source: http://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.html#_Toc398718021

    #[test]
    fn test_parse_connect() {
        // given
        let byte = 0x1Fu8;

        // when
        let packet_type = Parser::parse_packet_type(byte);

        // then
        assert_eq!(packet_type, ControlPacketType::Connect);
    }

    #[test]
    fn test_parse_connack() {
        // given
        let byte = 0x2Fu8;

        // when
        let packet_type = Parser::parse_packet_type(byte);

        // then
        assert_eq!(packet_type, ControlPacketType::ConnAck);
    }

    #[test]
    fn test_parse_publish() {
        // given
        let byte = 0x3Fu8;

        // when
        let packet_type = Parser::parse_packet_type(byte);

        // then
        assert_eq!(packet_type, ControlPacketType::Publish);
    }

    #[test]
    fn test_parse_puback() {
        // given
        let byte = 0x4Fu8;

        // when
        let packet_type = Parser::parse_packet_type(byte);

        // then
        assert_eq!(packet_type, ControlPacketType::PubAck);
    }

    #[test]
    fn test_parse_pubrec() {
        // given
        let byte = 0x5Fu8;

        // when
        let packet_type = Parser::parse_packet_type(byte);

        // then
        assert_eq!(packet_type, ControlPacketType::PubRec);
    }

    #[test]
    fn test_parse_pubrel() {
        // given
        let byte = 0x6Fu8;

        // when
        let packet_type = Parser::parse_packet_type(byte);

        // then
        assert_eq!(packet_type, ControlPacketType::PubRel);
    }

    #[test]
    fn test_parse_pubcomp() {
        // given
        let byte = 0x7Fu8;

        // when
        let packet_type = Parser::parse_packet_type(byte);

        // then
        assert_eq!(packet_type, ControlPacketType::PubComp);
    }

    #[test]
    fn test_parse_subscribe() {
        // given
        let byte = 0x8Fu8;

        // when
        let packet_type = Parser::parse_packet_type(byte);

        // then
        assert_eq!(packet_type, ControlPacketType::Subscribe);
    }

    #[test]
    fn test_parse_suback() {
        // given
        let byte = 0x9Fu8;

        // when
        let packet_type = Parser::parse_packet_type(byte);

        // then
        assert_eq!(packet_type, ControlPacketType::SubAck);
    }

    #[test]
    fn test_parse_unsubscribe() {
        // given
        let byte = 0xAFu8;

        // when
        let packet_type = Parser::parse_packet_type(byte);

        // then
        assert_eq!(packet_type, ControlPacketType::Unsubscribe);
    }

    #[test]
    fn test_parse_unsuback() {
        // given
        let byte = 0xBFu8;

        // when
        let packet_type = Parser::parse_packet_type(byte);

        // then
        assert_eq!(packet_type, ControlPacketType::UnsubAck);
    }

    #[test]
    fn test_parse_pingreq() {
        // given
        let byte = 0xCFu8;

        // when
        let packet_type = Parser::parse_packet_type(byte);

        // then
        assert_eq!(packet_type, ControlPacketType::PingReq);
    }

    #[test]
    fn test_parse_pingresp() {
        // given
        let byte = 0xDFu8;

        // when
        let packet_type = Parser::parse_packet_type(byte);

        // then
        assert_eq!(packet_type, ControlPacketType::PingResp);
    }

    #[test]
    fn test_parse_disconnect() {
        // given
        let byte = 0xEFu8;

        // when
        let packet_type = Parser::parse_packet_type(byte);

        // then
        assert_eq!(packet_type, ControlPacketType::Disconnect);
    }

    #[test]
    fn test_parse_reserved() {
        assert_eq!(
            Parser::parse_packet_type(0x0Fu8),
            ControlPacketType::Reserved0
        );
        assert_eq!(
            Parser::parse_packet_type(0xFFu8),
            ControlPacketType::Reserved15
        );
    }
}
