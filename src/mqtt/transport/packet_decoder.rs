use crate::mqtt::packets::puback::PubAckPacket;
use crate::mqtt::packets::pubcomp::PubCompPacket;
use crate::mqtt::packets::pubrec::PubRecPacket;
use crate::mqtt::packets::pubrel::PubRelPacket;
use crate::mqtt::packets::subscribe::SubscribePacket;
use crate::mqtt::packets::unsubscribe::UnsubscribePacket;
use crate::mqtt::packets::{
    ConnectPacket, ControlPacket, DisconnectPacket, ProtocolVersion, PublishPacket, QoS,
};
use crate::mqtt::packets::{
    PACKET_TYPE_CONNECT, PACKET_TYPE_DISCONNECT, PACKET_TYPE_PING_REQ, PACKET_TYPE_PUBLISH,
    PACKET_TYPE_PUB_ACK, PACKET_TYPE_PUB_COMP, PACKET_TYPE_PUB_REC, PACKET_TYPE_PUB_REL,
    PACKET_TYPE_SUBSCRIBE, PACKET_TYPE_UNSUBSCRIBE,
};
use crate::mqtt::subscription::Subscription;
use crate::mqtt::transport::mqtt_bytes_stream::MqttBytesReadStream;
use async_trait::async_trait;
use bitflags::bitflags;
use tokio::io::{Error, ErrorKind};

#[async_trait]
pub trait PacketDecoder {
    fn parse_fixed_header_flags(&mut self, flags: u8) -> Result<(), Error>;

    fn variable_header_size(&self) -> usize {
        0
    }

    async fn parse_variable_header(
        &mut self,
        _buffer: &mut MqttBytesReadStream,
    ) -> Result<usize, Error> {
        Ok(0)
    }

    async fn parse_payload(
        &mut self,
        _buffer: &mut MqttBytesReadStream,
        _remaining_length: u64,
    ) -> Result<usize, Error> {
        Ok(0)
    }
}

pub async fn decode_remaining_length(buffer: &mut MqttBytesReadStream) -> Result<u64, Error> {
    let mut remaining_length = 0u64;
    let mut multiplier = 1u64;

    loop {
        let byte = buffer.get_u8().await?;
        remaining_length += (byte & 127) as u64 * multiplier;
        multiplier *= 128;

        if multiplier > 128 * 128 * 128 {
            return Err(tokio::io::Error::new(
                ErrorKind::InvalidData,
                "Malformed remaining length multiplier",
            ));
        }

        let continuation_bit = byte & 128;
        if continuation_bit == 0 {
            break;
        }
    }

    Ok(remaining_length)
}

pub async fn read_packet(mqtt_stream: &mut MqttBytesReadStream) -> Result<ControlPacket, Error> {
    let first_byte = mqtt_stream.get_u8().await?;
    let packet_type = first_byte >> 4;
    let remaining_length = decode_remaining_length(mqtt_stream).await?;

    let packet = match packet_type {
        PACKET_TYPE_CONNECT => decode_connect(mqtt_stream, first_byte, remaining_length).await?,
        PACKET_TYPE_PUBLISH => decode_publish(mqtt_stream, first_byte, remaining_length).await?,
        PACKET_TYPE_PUB_ACK => decode_pub_ack(mqtt_stream, first_byte, remaining_length).await?,
        PACKET_TYPE_PUB_REC => decode_pub_rec(mqtt_stream, first_byte, remaining_length).await?,
        PACKET_TYPE_PUB_REL => decode_pub_rel(mqtt_stream, first_byte, remaining_length).await?,
        PACKET_TYPE_PUB_COMP => decode_pub_comp(mqtt_stream, first_byte, remaining_length).await?,
        PACKET_TYPE_SUBSCRIBE => {
            decode_subscribe(mqtt_stream, first_byte, remaining_length).await?
        }
        PACKET_TYPE_UNSUBSCRIBE => {
            decode_unsubscribe(mqtt_stream, first_byte, remaining_length).await?
        }
        PACKET_TYPE_PING_REQ => ControlPacket::PingReq,
        PACKET_TYPE_DISCONNECT => ControlPacket::Disconnect(DisconnectPacket::default()),
        _ => unimplemented!(),
    };

    Ok(packet)
}

async fn decode_connect(
    buffer: &mut MqttBytesReadStream,
    _first_byte: u8,
    _remaining_length: u64,
) -> Result<ControlPacket, Error> {
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

    // variable header
    let _protocol_name = buffer.get_string().await?;
    let _protocol_level = buffer.get_u8().await?;
    let connect_flags_byte = buffer.get_u8().await?;
    let connect_flags = ConnectFlags::from_bits_truncate(connect_flags_byte);

    let clean_session = connect_flags.contains(ConnectFlags::CLEAN_SESSION);

    let keep_alive_seconds = buffer.get_u16().await?;

    // payload
    let client_id = buffer.get_string().await?;

    let connect_packet = ConnectPacket::new(
        ProtocolVersion::Mqtt3,
        client_id,
        keep_alive_seconds,
        clean_session,
        None,
        None,
        None,
    );

    Ok(ControlPacket::Connect(connect_packet))
}

async fn decode_publish(
    buffer: &mut MqttBytesReadStream,
    first_byte: u8,
    mut remaining_length: u64,
) -> Result<ControlPacket, Error> {
    bitflags! {
        struct FixedHeaderFlags: u8 {
            const RETAIN =  0b00000001;
            const QOS_1 =   0b00000010;
            const QOS_2 =   0b00000100;
            const DUP =     0b00001000;
            const QOS = Self::QOS_1.bits | Self::QOS_2.bits;
        }
    }

    // fixed header
    let flags = FixedHeaderFlags::from_bits_truncate(first_byte);

    let dup = flags.contains(FixedHeaderFlags::DUP);
    let retain = flags.contains(FixedHeaderFlags::RETAIN);
    let qos = QoS::from_bits((flags & FixedHeaderFlags::QOS).bits >> 1);

    // variable header
    let topic = buffer.get_string().await?;
    remaining_length -= topic.len() as u64;
    remaining_length -= 2;

    let packet_id = if qos > QoS::AtMostOnce {
        remaining_length -= 2;
        Some(buffer.get_u16().await?)
    } else {
        None
    };

    // payload
    let body = buffer.get_bytes(remaining_length as usize).await?;

    let packet = PublishPacket::new(topic, body, qos, retain, packet_id, dup);
    Ok(ControlPacket::Publish(packet))
}

async fn decode_pub_ack(
    buffer: &mut MqttBytesReadStream,
    first_byte: u8,
    _remaining_length: u64,
) -> Result<ControlPacket, Error> {
    let packet_id = decode_packet_with_packet_id(buffer, first_byte, 0b01000000).await?;
    Ok(ControlPacket::PubAck(PubAckPacket::new(packet_id)))
}

async fn decode_pub_rec(
    buffer: &mut MqttBytesReadStream,
    first_byte: u8,
    _remaining_length: u64,
) -> Result<ControlPacket, Error> {
    let packet_id = decode_packet_with_packet_id(buffer, first_byte, 0b01010000).await?;
    Ok(ControlPacket::PubRec(PubRecPacket::new(packet_id)))
}

async fn decode_pub_rel(
    buffer: &mut MqttBytesReadStream,
    first_byte: u8,
    _remaining_length: u64,
) -> Result<ControlPacket, Error> {
    let packet_id = decode_packet_with_packet_id(buffer, first_byte, 0b01100010).await?;
    Ok(ControlPacket::PubRel(PubRelPacket::new(packet_id)))
}

async fn decode_pub_comp(
    buffer: &mut MqttBytesReadStream,
    first_byte: u8,
    _remaining_length: u64,
) -> Result<ControlPacket, Error> {
    let packet_id = decode_packet_with_packet_id(buffer, first_byte, 0b01110000).await?;
    Ok(ControlPacket::PubComp(PubCompPacket::new(packet_id)))
}

async fn decode_subscribe(
    buffer: &mut MqttBytesReadStream,
    first_byte: u8,
    mut remaining_length: u64,
) -> Result<ControlPacket, Error> {
    let packet_id = decode_packet_with_packet_id(buffer, first_byte, 0b10000010).await?;
    remaining_length -= 2;

    // payload

    if remaining_length == 0 {
        return Err(tokio::io::Error::new(
            ErrorKind::InvalidData,
            "Malformed payload",
        ));
    }

    let mut subscriptions = Vec::new();
    while remaining_length > 0 {
        let topic = buffer.get_string().await?;
        let qos = buffer.get_u8().await?;
        if qos > 2 {
            return Err(tokio::io::Error::new(
                ErrorKind::InvalidData,
                "Malformed QoS",
            ));
        }

        remaining_length -= topic.len() as u64 + 3u64 /* 2 topic length + QoS*/;

        let subscription = Subscription::new(topic, QoS::from_bits(qos));
        subscriptions.push(subscription);
    }

    let packet = SubscribePacket::new(packet_id, subscriptions);
    Ok(ControlPacket::Subscribe(packet))
}

async fn decode_unsubscribe(
    buffer: &mut MqttBytesReadStream,
    first_byte: u8,
    mut remaining_length: u64,
) -> Result<ControlPacket, Error> {
    let packet_id = decode_packet_with_packet_id(buffer, first_byte, 0b10100010).await?;
    remaining_length -= 2;

    if remaining_length == 0 {
        return Err(tokio::io::Error::new(
            ErrorKind::InvalidData,
            "Malformed payload",
        ));
    }

    let mut topics = Vec::new();
    while remaining_length > 0 {
        let topic = buffer.get_string().await?;

        remaining_length -= topic.len() as u64 + 2u64/* 2 topic length */;

        topics.push(topic);
    }

    let packet = UnsubscribePacket::new(packet_id, topics);
    Ok(ControlPacket::Unsubscribe(packet))
}

async fn decode_packet_with_packet_id(
    buffer: &mut MqttBytesReadStream,
    first_byte: u8,
    expected_first_byte: u8,
) -> Result<u16, Error> {
    validate_first_byte(first_byte, expected_first_byte)?;

    let packet_id = buffer.get_u16().await?;
    Ok(packet_id)
}

fn validate_first_byte(actual: u8, expected: u8) -> Result<(), Error> {
    if actual == expected {
        Ok(())
    } else {
        Err(tokio::io::Error::new(
            ErrorKind::InvalidData,
            "Malformed fixed header",
        ))
    }
}
