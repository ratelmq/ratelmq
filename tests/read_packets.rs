use tokio::io::AsyncWriteExt;
use tokio::net::{TcpListener, TcpStream};

use ratelmq::mqtt::packets::{ControlPacket, ProtocolVersion, QoS};
use ratelmq::mqtt::subscription::Subscription;
use ratelmq::mqtt::transport::mqtt_bytes_stream::MqttBytesReadStream;
use ratelmq::mqtt::transport::packet_decoder;

#[tokio::test]
async fn it_read_connect_min() {
    const DATA: &[u8] = &[
        0x10, 0x23, 0x00, 0x04, 0x4d, 0x51, 0x54, 0x54, 0x04, 0x02, 0x00, 0x3c, 0x00, 0x17, 0x6d,
        0x6f, 0x73, 0x71, 0x2d, 0x6e, 0x73, 0x36, 0x73, 0x7a, 0x33, 0x6b, 0x33, 0x6c, 0x62, 0x66,
        0x4d, 0x31, 0x49, 0x66, 0x62, 0x63, 0x52,
    ];

    let packet = read_packet(DATA).await;

    match packet {
        ControlPacket::Connect(connect) => {
            assert_eq!(connect.version, ProtocolVersion::Mqtt3);

            assert_eq!(connect.user_name, None);
            assert_eq!(connect.password, None);
            assert_eq!(connect.will_message, None);
            assert_eq!(connect.clean_session, true);

            assert_eq!(connect.keep_alive_seconds, 60);
            assert_eq!(connect.client_id, "mosq-ns6sz3k3lbfM1IfbcR");
        }
        _ => panic!("Invalid packet type"),
    };
}

#[tokio::test]
async fn it_read_disconnect() {
    const DATA: &[u8] = &[0xe0, 0x00];

    let packet = read_packet(DATA).await;

    match packet {
        ControlPacket::Disconnect(_) => {}
        _ => panic!("Invalid packet type"),
    };
}

#[tokio::test]
async fn it_read_publish_qos_0() {
    const DATA: &[u8] = &[
        0x30, 0x10, 0x00, 0x05, 0x61, 0x2f, 0x62, 0x2f, 0x63, 0x74, 0x65, 0x73, 0x74, 0x20, 0x62,
        0x6f, 0x64, 0x79,
    ];

    let packet = read_packet(DATA).await;

    match packet {
        ControlPacket::Publish(publish) => {
            assert_eq!(publish.dup, false);
            assert_eq!(publish.qos, QoS::AtMostOnce);
            assert_eq!(publish.retain, false);
            assert_eq!(publish.packet_id, None);
            assert_eq!(publish.topic, "a/b/c");
            assert_eq!(publish.body, "test body");
        }
        _ => panic!("Invalid packet type"),
    };
}

#[tokio::test]
async fn it_read_publish_qos_greater_than_0() {
    const DATA: &[u8] = &[
        0x3d, 0x12, 0x00, 0x05, 0x61, 0x2f, 0x62, 0x2f, 0x63, 0x12, 0x23, 0x74, 0x65, 0x73, 0x74,
        0x20, 0x62, 0x6f, 0x64, 0x79,
    ];

    let packet = read_packet(DATA).await;

    match packet {
        ControlPacket::Publish(publish) => {
            assert_eq!(publish.dup, true);
            assert_eq!(publish.retain, true);
            assert_eq!(publish.qos, QoS::ExactlyOnce);
            assert_eq!(publish.topic, "a/b/c");
            assert_eq!(publish.packet_id, Some(0x1223));
            assert_eq!(publish.body, "test body");
        }
        _ => panic!("Invalid packet type"),
    };
}

#[tokio::test]
async fn it_read_pub_ack() {
    const DATA: &[u8] = &[0x40, 0x02, 0x00, 0x02];

    let packet = read_packet(DATA).await;

    match packet {
        ControlPacket::PubAck(pub_ack) => {
            assert_eq!(pub_ack.packet_id, 0x02);
        }
        _ => panic!("Invalid packet type"),
    };
}

#[tokio::test]
async fn it_read_pub_rec() {
    const DATA: &[u8] = &[0x50, 0x02, 0x12, 0x34];

    let packet = read_packet(DATA).await;

    match packet {
        ControlPacket::PubRec(pub_rec) => {
            assert_eq!(pub_rec.packet_id, 0x1234);
        }
        _ => panic!("Invalid packet type"),
    };
}

#[tokio::test]
async fn it_read_pub_rel() {
    const DATA: &[u8] = &[0x62, 0x02, 0x00, 0x67];

    let packet = read_packet(DATA).await;

    match packet {
        ControlPacket::PubRel(pub_rel) => {
            assert_eq!(pub_rel.packet_id, 0x67);
        }
        _ => panic!("Invalid packet type"),
    };
}

#[tokio::test]
async fn it_read_pub_comp() {
    const DATA: &[u8] = &[0x70, 0x02, 0xcd, 0x12];

    let packet = read_packet(DATA).await;

    match packet {
        ControlPacket::PubComp(pub_comp) => {
            assert_eq!(pub_comp.packet_id, 0xcd12);
        }
        _ => panic!("Invalid packet type"),
    };
}

#[tokio::test]
async fn it_read_subscribe_one() {
    const DATA: &[u8] = &[
        0x82, 0x0a, 0x00, 0x01, 0x00, 0x05, 0x61, 0x2f, 0x62, 0x2f, 0x63, 0x00,
    ];

    let packet = read_packet(DATA).await;

    match packet {
        ControlPacket::Subscribe(subscribe) => {
            assert_eq!(subscribe.packet_id, 1);
            assert_eq!(
                subscribe.subscriptions,
                vec![Subscription::new("a/b/c".to_string(), QoS::AtMostOnce)]
            );
        }
        _ => panic!("Invalid packet type"),
    };
}

#[tokio::test]
async fn it_read_subscribe_many() {
    const DATA: &[u8] = &[
        0x82, 0x1a, 0x00, 0x01, 0x00, 0x05, 0x61, 0x2f, 0x62, 0x2f, 0x63, 0x00, 0x00, 0x05, 0x7a,
        0x2f, 0x78, 0x2f, 0x63, 0x01, 0x00, 0x05, 0x71, 0x2f, 0x77, 0x2f, 0x65, 0x02,
    ];

    let packet = read_packet(DATA).await;

    match packet {
        ControlPacket::Subscribe(subscribe) => {
            assert_eq!(subscribe.packet_id, 1);
            assert_eq!(
                subscribe.subscriptions,
                vec![
                    Subscription::new("a/b/c".to_string(), QoS::AtMostOnce),
                    Subscription::new("z/x/c".to_string(), QoS::AtLeastOnce),
                    Subscription::new("q/w/e".to_string(), QoS::ExactlyOnce)
                ]
            );
        }
        _ => panic!("Invalid packet type"),
    };
}

#[tokio::test]
async fn it_read_unsubscribe_one() {
    const DATA: &[u8] = &[
        0xa2, 0x09, 0x00, 0x02, 0x00, 0x05, 0x61, 0x2f, 0x62, 0x2f, 0x63,
    ];

    let packet = read_packet(DATA).await;

    match packet {
        ControlPacket::Unsubscribe(unsubscribe) => {
            assert_eq!(unsubscribe.packet_id, 2);
            assert_eq!(unsubscribe.topics, vec!["a/b/c".to_string()]);
        }
        _ => panic!("Invalid packet type"),
    };
}

#[tokio::test]
async fn it_read_unsubscribe_many() {
    const DATA: &[u8] = &[
        0xa2, 0x17, 0x00, 0x02, 0x00, 0x05, 0x61, 0x2f, 0x62, 0x2f, 0x63, 0x00, 0x05, 0x7a, 0x2f,
        0x78, 0x2f, 0x63, 0x00, 0x05, 0x71, 0x2f, 0x77, 0x2f, 0x65,
    ];

    let packet = read_packet(DATA).await;

    match packet {
        ControlPacket::Unsubscribe(unsubscribe) => {
            assert_eq!(unsubscribe.packet_id, 2);
            assert_eq!(
                unsubscribe.topics,
                vec![
                    "a/b/c".to_string(),
                    "z/x/c".to_string(),
                    "q/w/e".to_string()
                ]
            );
        }
        _ => panic!("Invalid packet type"),
    };
}

#[tokio::test]
async fn it_read_ping_req() {
    const DATA: &[u8] = &[0xc0, 0x00];

    let packet = read_packet(DATA).await;

    match packet {
        ControlPacket::PingReq => {
            // nothing to assert
        }
        _ => panic!("Invalid packet type"),
    };
}

async fn read_packet(data: &[u8]) -> ControlPacket {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();

    let mut client = TcpStream::connect(listener.local_addr().unwrap())
        .await
        .unwrap();

    let (server, _) = listener.accept().await.unwrap();
    client.write(data).await.unwrap();

    let (rx, _) = server.into_split();
    let mut mqtt_buffer = MqttBytesReadStream::new(4096, rx);

    packet_decoder::read_packet(&mut mqtt_buffer).await.unwrap()
}
