use async_trait::async_trait;
use bitflags::bitflags;
use tokio::io::Error;

use crate::mqtt::packets::puback::PubAckPacket;
use crate::mqtt::packets::pubcomp::PubCompPacket;
use crate::mqtt::packets::pubrec::PubRecPacket;
use crate::mqtt::packets::pubrel::PubRelPacket;
use crate::mqtt::packets::suback::SubAckPacket;
use crate::mqtt::packets::unsuback::UnSubAckPacket;
use crate::mqtt::packets::{
    ConnAckPacket, ControlPacket, PublishPacket, QoS, PACKET_TYPE_CONN_ACK, PACKET_TYPE_PING_RESP,
    PACKET_TYPE_PUBLISH, PACKET_TYPE_PUB_ACK, PACKET_TYPE_PUB_COMP, PACKET_TYPE_PUB_REC,
    PACKET_TYPE_PUB_REL, PACKET_TYPE_SUB_ACK, PACKET_TYPE_UNSUB_ACK,
};
use crate::mqtt::transport::mqtt_bytes_stream::{MqttBytesStream, MqttBytesWriteStream};

#[async_trait]
pub trait PacketEncoder {
    async fn encode_fixed_header(&self, buffer: &mut MqttBytesStream) -> Result<(), Error>;
    async fn encode_variable_header(&self, _buffer: &mut MqttBytesStream) -> Result<(), Error> {
        Ok(())
    }
    async fn encode_body(&self, _buffer: &mut MqttBytesStream) -> Result<(), Error> {
        Ok(())
    }
}

pub async fn encode_remaining_length(
    mut remaining_length: u64,
    buffer: &mut MqttBytesWriteStream,
) -> Result<(), Error> {
    loop {
        let mut encoded_byte: u8 = (remaining_length % 128) as u8;
        remaining_length /= 128;

        if remaining_length > 0 {
            encoded_byte |= 128;
        }

        buffer.put_u8(encoded_byte).await?;

        if remaining_length == 0 {
            break;
        }
    }
    Ok(())
}

pub async fn write_packet(
    mqtt_stream: &mut MqttBytesWriteStream,
    packet: ControlPacket,
) -> Result<(), Error> {
    match packet {
        ControlPacket::ConnAck(conn_ack) => write_conn_ack(mqtt_stream, conn_ack).await?,
        ControlPacket::Publish(publish) => write_publish(mqtt_stream, publish).await?,
        ControlPacket::PubAck(pub_ack) => write_pub_ack(mqtt_stream, pub_ack).await?,
        ControlPacket::PubRec(pub_rec) => write_pub_rec(mqtt_stream, pub_rec).await?,
        ControlPacket::PubRel(pub_rel) => write_pub_rel(mqtt_stream, pub_rel).await?,
        ControlPacket::PubComp(pub_comp) => write_pub_comp(mqtt_stream, pub_comp).await?,
        ControlPacket::SubAck(sub_ack) => write_sub_ack(mqtt_stream, sub_ack).await?,
        ControlPacket::UnsubAck(unsub_ack) => write_unsub_ack(mqtt_stream, unsub_ack).await?,
        ControlPacket::PingResp => write_ping_resp(mqtt_stream).await?,
        _ => unimplemented!(),
    };

    mqtt_stream.finish_packet().await?;

    Ok(())
}

async fn write_conn_ack(
    buffer: &mut MqttBytesWriteStream,
    packet: ConnAckPacket,
) -> Result<(), Error> {
    // fixed header
    buffer.put_u8(PACKET_TYPE_CONN_ACK << 4).await?;

    const REMAINING_LENGTH: u64 = 2;
    encode_remaining_length(REMAINING_LENGTH, buffer).await?;

    // variable header
    buffer.put_u8(packet.session_present as u8).await?;
    buffer.put_u8(packet.return_code.clone() as u8).await?;

    Ok(())
}

async fn write_publish(
    buffer: &mut MqttBytesWriteStream,
    packet: PublishPacket,
) -> Result<(), Error> {
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
    let mut first_byte = PACKET_TYPE_PUBLISH << 4;
    if packet.dup {
        first_byte |= FixedHeaderFlags::DUP.bits;
    }
    first_byte |= (packet.message.qos as u8) << 1;
    if packet.message.retain {
        first_byte |= FixedHeaderFlags::RETAIN.bits;
    }

    buffer.put_u8(first_byte).await?;

    let mut remaining_length: u64 = (packet.message.topic.len() + 2) as u64;

    if packet.message.qos > QoS::AtMostOnce {
        remaining_length += 2 /* packet id */;
    }

    remaining_length += packet.message.payload.len() as u64;

    encode_remaining_length(remaining_length, buffer).await?;

    // variable header
    buffer.put_string(packet.message.topic.as_str()).await?;

    if let Some(packet_id) = packet.packet_id {
        buffer.put_u16(packet_id).await?;
    }

    // payload

    // todo: refactor to eliminated clone
    buffer.put_bytes(packet.message.payload.clone()).await?;
    Ok(())
}

async fn write_pub_ack(
    buffer: &mut MqttBytesWriteStream,
    packet: PubAckPacket,
) -> Result<(), Error> {
    write_packet_with_packet_id(buffer, PACKET_TYPE_PUB_ACK << 4, packet.packet_id).await?;
    Ok(())
}

async fn write_pub_rec(
    buffer: &mut MqttBytesWriteStream,
    packet: PubRecPacket,
) -> Result<(), Error> {
    write_packet_with_packet_id(buffer, PACKET_TYPE_PUB_REC << 4, packet.packet_id).await?;
    Ok(())
}

async fn write_pub_rel(
    buffer: &mut MqttBytesWriteStream,
    packet: PubRelPacket,
) -> Result<(), Error> {
    let mut first_byte = PACKET_TYPE_PUB_REL << 4;
    first_byte |= 0b10;
    write_packet_with_packet_id(buffer, first_byte, packet.packet_id).await?;
    Ok(())
}

async fn write_pub_comp(
    buffer: &mut MqttBytesWriteStream,
    packet: PubCompPacket,
) -> Result<(), Error> {
    write_packet_with_packet_id(buffer, PACKET_TYPE_PUB_COMP << 4, packet.packet_id).await?;
    Ok(())
}

async fn write_sub_ack(
    buffer: &mut MqttBytesWriteStream,
    packet: SubAckPacket,
) -> Result<(), Error> {
    // fixed header
    buffer.put_u8(PACKET_TYPE_SUB_ACK << 4).await?;

    let return_codes = packet.return_codes;

    let remaining_length: u64 = 2 /* packet id */ + return_codes.len() as u64;
    encode_remaining_length(remaining_length, buffer).await?;

    // variable header
    buffer.put_u16(packet.packet_id).await?;

    // payload
    // todo: refactor to eliminated clone
    for return_code in &return_codes {
        buffer.put_u8(return_code.clone() as u8).await?;
    }

    Ok(())
}

async fn write_unsub_ack(
    buffer: &mut MqttBytesWriteStream,
    packet: UnSubAckPacket,
) -> Result<(), Error> {
    // fixed header
    write_packet_with_packet_id(buffer, PACKET_TYPE_UNSUB_ACK << 4, packet.packet_id).await?;

    Ok(())
}

async fn write_ping_resp(buffer: &mut MqttBytesWriteStream) -> Result<(), Error> {
    // fixed header
    buffer.put_u8(PACKET_TYPE_PING_RESP << 4).await?;

    const REMAINING_LENGTH: u64 = 0;
    encode_remaining_length(REMAINING_LENGTH, buffer).await?;

    Ok(())
}

async fn write_packet_with_packet_id(
    buffer: &mut MqttBytesWriteStream,
    first_byte: u8,
    packet_id: u16,
) -> Result<(), Error> {
    // fixed header
    buffer.put_u8(first_byte).await?;

    const REMAINING_LENGTH: u64 = 2;
    encode_remaining_length(REMAINING_LENGTH, buffer).await?;

    // variable header
    buffer.put_u16(packet_id).await?;

    Ok(())
}
