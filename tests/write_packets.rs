use bytes::BytesMut;
use tokio::io::AsyncReadExt;
use tokio::net::{TcpListener, TcpStream};

use ratelmq::mqtt::connection::Connection;
use ratelmq::mqtt::packets::ping_resp::PingRespPacket;
use ratelmq::mqtt::packets::puback::PubAckPacket;
use ratelmq::mqtt::packets::pubcomp::PubCompPacket;
use ratelmq::mqtt::packets::pubrec::PubRecPacket;
use ratelmq::mqtt::packets::pubrel::PubRelPacket;
use ratelmq::mqtt::packets::suback::SubAckPacket;
use ratelmq::mqtt::packets::unsuback::UnSubAckPacket;
use ratelmq::mqtt::packets::{ConnAckPacket, PublishPacket, QoS};
use ratelmq::mqtt::transport::packet_encoder::PacketEncoder;

#[tokio::test]
async fn it_write_connack() {
    let connack = ConnAckPacket::default();

    let data = write_packet(&connack).await;
    assert_bytes(data, vec![0x20, 0x02, 0x00, 0x00])
}

#[tokio::test]
async fn it_write_publish_qos_0() {
    const EXPECTED_DATA: &[u8] = &[
        0x30, 0x10, 0x00, 0x05, 0x61, 0x2f, 0x62, 0x2f, 0x63, 0x74, 0x65, 0x73, 0x74, 0x20, 0x62,
        0x6f, 0x64, 0x79,
    ];

    let mut publish = PublishPacket::default();
    publish.qos = QoS::AtMostOnce;
    publish.dup = false;
    publish.topic = "a/b/c".to_string();
    publish.body = BytesMut::from("test body");

    let data = write_packet(&publish).await;

    assert_bytes(data, EXPECTED_DATA.to_vec())
}

#[tokio::test]
async fn it_write_publish_qos_greater_than_0() {
    const EXPECTED_DATA: &[u8] = &[
        0x3d, 0x12, 0x00, 0x05, 0x61, 0x2f, 0x62, 0x2f, 0x63, 0x12, 0x23, 0x74, 0x65, 0x73, 0x74,
        0x20, 0x62, 0x6f, 0x64, 0x79,
    ];

    let mut publish = PublishPacket::default();
    publish.dup = true;
    publish.qos = QoS::ExactlyOnce;
    publish.retain = true;
    publish.packet_id = Some(0x1223);
    publish.topic = "a/b/c".to_string();
    publish.body = BytesMut::from("test body");

    let data = write_packet(&publish).await;

    assert_bytes(data, EXPECTED_DATA.to_vec())
}

#[tokio::test]
async fn it_write_puback() {
    let mut puback = PubAckPacket::default();
    puback.packet_id = 0x02;

    let data = write_packet(&puback).await;

    assert_bytes(data, vec![0x40, 0x02, 0x00, 0x02])
}

#[tokio::test]
async fn it_write_pubrec() {
    let mut pubrec = PubRecPacket::default();
    pubrec.packet_id = 0x1234;

    let data = write_packet(&pubrec).await;

    assert_bytes(data, vec![0x50, 0x02, 0x12, 0x34])
}

#[tokio::test]
async fn it_write_pubrel() {
    let mut pubrel = PubRelPacket::default();
    pubrel.packet_id = 0x67;

    let data = write_packet(&pubrel).await;

    assert_bytes(data, vec![0x62, 0x02, 0x00, 0x67])
}

#[tokio::test]
async fn it_write_pubcomp() {
    let mut pubcomp = PubCompPacket::default();
    pubcomp.packet_id = 0xcd12;

    let data = write_packet(&pubcomp).await;

    assert_bytes(data, vec![0x70, 0x02, 0xcd, 0x12])
}

#[tokio::test]
async fn it_write_suback() {
    let mut suback = SubAckPacket::default();
    suback.packet_id = 0xa3c9;

    let data = write_packet(&suback).await;

    assert_bytes(data, vec![0x90, 0x02, 0xa3, 0xc9])
}

#[tokio::test]
async fn it_write_unsuback() {
    let mut unsuback = UnSubAckPacket::default();
    unsuback.packet_id = 6;

    let data = write_packet(&unsuback).await;

    assert_bytes(data, vec![0xb0, 0x02, 0x00, 0x06])
}

#[tokio::test]
async fn it_write_ping_resp() {
    let ping_resp = PingRespPacket::default();

    let data = write_packet(&ping_resp).await;

    assert_bytes(data, vec![0xd0, 0x00])
}

async fn write_packet<T>(packet: &T) -> BytesMut
where
    T: PacketEncoder + Sync,
{
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();

    let mut client = TcpStream::connect(listener.local_addr().unwrap())
        .await
        .unwrap();

    let (server, _) = listener.accept().await.unwrap();

    let mut connection = Connection::new(server, 8096);
    connection.write_packet(packet).await.unwrap();

    let mut buffer = BytesMut::with_capacity(1024);
    client.read_buf(&mut buffer).await.unwrap();
    buffer
}

fn assert_bytes(actual: BytesMut, expected: Vec<u8>) {
    let actual_vec = actual.to_vec();
    assert_eq!(
        actual_vec, expected,
        "Actual: {:02X?}\nExpected: {:02X?}",
        actual_vec, expected
    );
}
