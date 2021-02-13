use bytes::BytesMut;
use tokio::io::AsyncReadExt;
use tokio::net::{TcpListener, TcpStream};

use ratelmq::mqtt::packets::puback::PubAckPacket;
use ratelmq::mqtt::packets::pubcomp::PubCompPacket;
use ratelmq::mqtt::packets::pubrec::PubRecPacket;
use ratelmq::mqtt::packets::pubrel::PubRelPacket;
use ratelmq::mqtt::packets::suback::{SubAckPacket, SubAckReturnCode};
use ratelmq::mqtt::packets::unsuback::UnSubAckPacket;
use ratelmq::mqtt::packets::{ConnAckPacket, ControlPacket, PublishPacket, QoS};
use ratelmq::mqtt::transport::mqtt_bytes_stream::MqttBytesWriteStream;
use ratelmq::mqtt::transport::packet_encoder;

#[tokio::test]
async fn it_write_conn_ack() {
    let conn_ack = ConnAckPacket::default();

    let data = write_packet(ControlPacket::ConnAck(conn_ack)).await;
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

    let data = write_packet(ControlPacket::Publish(publish)).await;

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

    let data = write_packet(ControlPacket::Publish(publish)).await;

    assert_bytes(data, EXPECTED_DATA.to_vec())
}

#[tokio::test]
async fn it_write_pub_ack() {
    let mut pub_ack = PubAckPacket::default();
    pub_ack.packet_id = 0x02;

    let data = write_packet(ControlPacket::PubAck(pub_ack)).await;

    assert_bytes(data, vec![0x40, 0x02, 0x00, 0x02])
}

#[tokio::test]
async fn it_write_pub_rec() {
    let mut pub_rec = PubRecPacket::default();
    pub_rec.packet_id = 0x1234;

    let data = write_packet(ControlPacket::PubRec(pub_rec)).await;

    assert_bytes(data, vec![0x50, 0x02, 0x12, 0x34])
}

#[tokio::test]
async fn it_write_pubrel() {
    let mut pub_rel = PubRelPacket::default();
    pub_rel.packet_id = 0x67;

    let data = write_packet(ControlPacket::PubRel(pub_rel)).await;

    assert_bytes(data, vec![0x62, 0x02, 0x00, 0x67])
}

#[tokio::test]
async fn it_write_pub_comp() {
    let mut pub_comp = PubCompPacket::default();
    pub_comp.packet_id = 0xcd12;

    let data = write_packet(ControlPacket::PubComp(pub_comp)).await;

    assert_bytes(data, vec![0x70, 0x02, 0xcd, 0x12])
}

#[tokio::test]
async fn it_write_sub_ack() {
    let mut sub_ack = SubAckPacket::default();
    sub_ack.packet_id = 0xa3c9;
    sub_ack.return_codes = vec![
        SubAckReturnCode::Failure,
        SubAckReturnCode::SuccessQoS0,
        SubAckReturnCode::SuccessQoS1,
        SubAckReturnCode::SuccessQoS2,
    ];

    let data = write_packet(ControlPacket::SubAck(sub_ack)).await;

    assert_bytes(data, vec![0x90, 0x06, 0xa3, 0xc9, 0x80, 0x00, 0x01, 0x02])
}

#[tokio::test]
async fn it_write_unsub_ack() {
    let mut unsub_ack = UnSubAckPacket::default();
    unsub_ack.packet_id = 6;

    let data = write_packet(ControlPacket::UnsubAck(unsub_ack)).await;

    assert_bytes(data, vec![0xb0, 0x02, 0x00, 0x06])
}

#[tokio::test]
async fn it_write_ping_resp() {
    let data = write_packet(ControlPacket::PingResp).await;

    assert_bytes(data, vec![0xd0, 0x00])
}

async fn write_packet(packet: ControlPacket) -> BytesMut {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();

    let mut client = TcpStream::connect(listener.local_addr().unwrap())
        .await
        .unwrap();

    let (server, _) = listener.accept().await.unwrap();

    let (_, tx) = server.into_split();
    let mut mqtt_buffer = MqttBytesWriteStream::new(4096, tx);

    packet_encoder::write_packet(&mut mqtt_buffer, packet)
        .await
        .unwrap();

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
