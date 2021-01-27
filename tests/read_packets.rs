use tokio::io::AsyncWriteExt;
use tokio::net::{TcpListener, TcpStream};

use ratelmq::mqtt::connection::Connection;
use ratelmq::mqtt::packets::{ControlPacket, ProtocolVersion, QoS};
use ratelmq::mqtt::subscription::Subscription;

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
            assert_eq!(connect.will_retain, false);
            assert_eq!(connect.will_qos, QoS::AtMostOnce);
            assert_eq!(connect.will_flag, false);
            assert_eq!(connect.will_topic, None);
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
async fn it_read_publish_min() {
    const DATA: &[u8] = &[
        0x30, 0x10, 0x00, 0x05, 0x61, 0x2f, 0x62, 0x2f, 0x63, 0x74, 0x65, 0x73, 0x74, 0x20, 0x62,
        0x6f, 0x64, 0x79,
    ];

    let packet = read_packet(DATA).await;

    match packet {
        ControlPacket::Publish(publish) => {
            assert_eq!(publish.topic, "a/b/c");
            assert_eq!(publish.body, "test body");
            assert_eq!(publish.qos, QoS::AtMostOnce);
            assert_eq!(publish.retain, false);
        }
        _ => panic!("Invalid packet type"),
    };
}

#[tokio::test]
async fn it_read_publish_full() {
    const DATA: &[u8] = &[
        0x3d, 0x10, 0x00, 0x05, 0x61, 0x2f, 0x62, 0x2f, 0x63, 0x74, 0x65, 0x73, 0x74, 0x20, 0x62,
        0x6f, 0x64, 0x79,
    ];

    let packet = read_packet(DATA).await;

    match packet {
        ControlPacket::Publish(publish) => {
            assert_eq!(publish.topic, "a/b/c");
            assert_eq!(publish.body, "test body");
            assert_eq!(publish.qos, QoS::ExactlyOnce);
            assert_eq!(publish.retain, true);
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
async fn it_read_ping_req() {
    const DATA: &[u8] = &[0xc0, 0x00];

    let packet = read_packet(DATA).await;

    match packet {
        ControlPacket::PingReq(_) => {
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

    let mut connection = Connection::new(server, 8096);
    connection.read_packet().await.unwrap()
}
