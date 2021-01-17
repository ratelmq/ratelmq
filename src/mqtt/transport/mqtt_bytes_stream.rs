use std::str::Utf8Error;

use bytes::{Buf, BufMut, BytesMut};
use tokio::io::{AsyncReadExt, AsyncWriteExt, Error, ErrorKind};
use tokio::net::TcpStream;

pub struct MqttBytesStream {
    read_buffer: BytesMut,
    write_buffer: BytesMut,
    tcp_stream: TcpStream,
}

impl MqttBytesStream {
    pub fn new(read_buffer_size: usize, write_buffer_size: usize, tcp_stream: TcpStream) -> MqttBytesStream {
        MqttBytesStream {
            read_buffer: BytesMut::with_capacity(read_buffer_size),
            write_buffer: BytesMut::with_capacity(write_buffer_size),
            tcp_stream,
        }
    }

    pub fn tcp_stream(&self) -> &TcpStream {
        self.tcp_stream()
    }
}

impl MqttBytesStream {
    pub async fn get_u8(&mut self) -> Result<u8, Error> {
        self.wait_for_data(1).await?;

        Ok(self.read_buffer.get_u8())
    }

    pub async fn get_u16(&mut self) -> Result<u16, Error> {
        self.wait_for_data(2).await?;

        Ok(self.read_buffer.get_u16())
    }

    pub async fn get_string(&mut self) -> Result<String, Error> {
        println!("Parsing string size");
        let string_size = self.get_u16().await? as usize;
        println!("String size: {} ({:#04x})", string_size, string_size);
        println!("Parsing string buf");
        let str_buf = self.get_bytes(string_size).await?;

        let str = std::str::from_utf8(str_buf.as_ref()).map_err(|e| {
            Error::new(
                ErrorKind::InvalidData,
                format!(
                    "Invalid UTF8 String: error at position {}",
                    e.valid_up_to() + 1
                ),
            )
        })?;

        // println!(
        //     "\tparsed string: {:#?}, of length: {:#?}",
        //     str, &string_size
        // );

        Ok(String::from(str))
    }

    pub async fn get_bytes(&mut self, size: usize) -> Result<BytesMut, Error> {
        let mut bytes = BytesMut::with_capacity(size);

        let mut remaining_length = size;

        while remaining_length > 0 {
            // todo: use clamp when stabilized
            let wait_for_bytes = {
                if remaining_length > self.read_buffer.capacity() {
                    self.read_buffer.capacity()
                } else {
                    remaining_length
                }
            };

            self.wait_for_data(wait_for_bytes).await?;
            bytes.put(self.read_buffer.split_to(wait_for_bytes));
            remaining_length -= wait_for_bytes;
        }

        Ok(bytes)
    }

    async fn wait_for_data(&mut self, bytes: usize) -> Result<(), Error> {
        while self.read_buffer.len() < bytes {
            self.tcp_stream.read_buf(&mut self.read_buffer).await?;
        }

        Ok(())
    }
}

impl MqttBytesStream {
    pub async fn put_u8(&mut self, n: u8) -> Result<(), Error> {
        self.write_buffer_if_too_small(1).await?;

        self.write_buffer.put_u8(n);
        Ok(())
    }

    pub async fn put_u16(&mut self, n: u16) -> Result<(), Error> {
        self.write_buffer_if_too_small(2).await?;

        self.write_buffer.put_u16(n);
        Ok(())
    }

    pub async fn put_string(&mut self, string: &String) -> Result<(), Error> {
        self.put_u16(string.len() as u16).await?;
        self.put_bytes(BytesMut::from(string.as_bytes())).await?;

        Ok(())
    }

    pub async fn put_bytes(&mut self, mut bytes: BytesMut) -> Result<(), Error> {
        let mut remaining_size = bytes.len();

        while remaining_size > 0 {
            let bytes_to_write = {
                if remaining_size > self.write_buffer.capacity() {
                    self.write_buffer.capacity()
                } else {
                    remaining_size
                }
            };
            self.write_buffer_if_too_small(bytes_to_write).await?;

            self.write_buffer.put(bytes.split_to(bytes_to_write));

            remaining_size -= bytes_to_write;
        }

        Ok(())
    }

    pub async fn finish_packet(&mut self) -> Result<(), Error> {
        println!("Finished packet");
        self.tcp_stream.write_buf(&mut self.write_buffer).await?;
        // self.tcp_stream.flush().await?;
        Ok(())
    }

    async fn write_buffer_if_too_small(&mut self, size: usize) -> Result<(), Error> {
        if self.write_buffer.len() + size >= self.write_buffer.capacity() {
            self.tcp_stream.write_buf(&mut self.write_buffer).await?;
        }

        Ok(())
    }
}
