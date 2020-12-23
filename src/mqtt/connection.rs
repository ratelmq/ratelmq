use bytes::{Buf, BytesMut};
use tokio::io::{AsyncWriteExt, Error, ErrorKind};
use tokio::net::TcpStream;
use tokio::prelude::io::AsyncReadExt;

use crate::mqtt::codec::{Decoder, Encoder, MqttCodec};
use crate::mqtt::packets::{ControlPacket, ControlPacketType};
use crate::mqtt::parser::Parser;

pub struct Connection {
    stream: TcpStream,
    read_buffer: BytesMut,
    write_buffer: BytesMut,
    codec: MqttCodec,
}

impl Connection {
    pub fn new(stream: TcpStream, buffer_size: usize) -> Connection {
        Connection {
            stream,
            read_buffer: BytesMut::with_capacity(buffer_size),
            write_buffer: BytesMut::with_capacity(buffer_size),
            codec: MqttCodec::new(),
        }
    }

    pub async fn read_frame(&mut self) -> Result<Option<ControlPacket>, Error> {
        println!("Reading frame");
        loop {
            if let Some(frame) = self.codec.parse_frame(&mut self.read_buffer)? {
                println!("Returning frame!");
                return Ok(Some(frame));
            }

            let read_bytes_count = self.stream.read_buf(&mut self.read_buffer).await?;
            println!("Read {} bytes", read_bytes_count);

            let connection_closed = read_bytes_count == 0; // end of file
            if connection_closed {
                let clean_shutdown = self.read_buffer.is_empty();
                return if clean_shutdown {
                    Ok(None)
                } else {
                    // closed while sending
                    Err(tokio::io::Error::new(
                        ErrorKind::ConnectionReset,
                        "connection reset by peer",
                    ))
                };
            }
        }
    }

    pub async fn write_frame(&mut self, packet: &ControlPacket) -> Result<(), Error> {
        println!("Writing frame");
        self.codec.encode(packet, &mut self.write_buffer);
        self.stream.write_buf(&mut self.write_buffer).await?;
        // self.stream.flush().await?;
        Ok(())
    }
}
