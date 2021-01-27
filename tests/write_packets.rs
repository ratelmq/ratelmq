use bytes::{Bytes, BytesMut};
use ratelmq::mqtt::connection::Connection;
use ratelmq::mqtt::transport::packet_encoder::PacketEncoder;
use ratelmq::mqtt::packets::{ConnAckPacket, ControlPacket, ProtocolVersion, QoS};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

#[tokio::test]
async fn it_write_connack() {
    let connack = ConnAckPacket::default();

    let data = write_packet(&connack).await;
    assert_eq!(data, vec![0x20, 0x02, 0x00, 0x00])
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
