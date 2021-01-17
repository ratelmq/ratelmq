use std::str;

use bitflags::_core::str::Utf8Error;
use bytes::{Buf, BufMut, BytesMut};
use tokio::io::{Error, ErrorKind};

use crate::mqtt::packets::connect::ConnectPacket;
use crate::mqtt::packets::ControlPacketType::Connect;
use crate::mqtt::packets::{ControlPacket, ControlPacketType, FIXED_HEADER_MAX_SIZE};
use crate::mqtt::parser::Parser;
use crate::mqtt::transport::packet_encoder::PacketEncoder;
use crate::mqtt::utils::{parse_string, parse_u16};

#[derive(Debug, PartialEq)]
enum ParserState {
    FixedHeader,
    VariableHeader,
    Payload,
}

pub struct MqttCodec {
    state: ParserState,
    packet: ControlPacket,
    packet_type: ControlPacketType,
    remaining_length: u64,
    remaining_length_multiplier: u64,
}

pub trait Decoder {
    fn parse_frame(&mut self, buffer: &mut BytesMut) -> Result<Option<ControlPacket>, Error>;
}

pub trait Encoder {
    fn encode(&mut self, packet: &ControlPacket, buffer: &mut BytesMut) -> Result<(), Error>;
}

impl MqttCodec {
    pub fn new() -> MqttCodec {
        MqttCodec {
            state: ParserState::FixedHeader,
            packet: ControlPacket::Test(),
            packet_type: ControlPacketType::Reserved0,
            remaining_length: 0,
            remaining_length_multiplier: 1,
        }
    }

    fn parse_fixed_header(&mut self, buffer: &mut BytesMut) -> Result<(), Error> {
        println!("Parsing fixed header, buffer len: {}", buffer.len());

        self.packet_type = Parser::parse_packet_type(buffer[0]);
        self.packet = MqttCodec::create_packet(&self.packet_type);

        buffer.advance(1);
        self.remaining_length = buffer[1].into();

        loop {
            let byte = buffer[0];
            buffer.advance(1);

            self.remaining_length += (byte & 127) as u64 * self.remaining_length_multiplier;
            self.remaining_length_multiplier *= 128;

            if self.remaining_length_multiplier > 128 * 128 * 128 {
                return Err(tokio::io::Error::new(
                    ErrorKind::InvalidData,
                    "Malformed multiplier",
                ));
            }

            let continuation_bit = byte & 128;
            if continuation_bit == 0 {
                self.state = ParserState::VariableHeader;
                break;
            }
        }

        println!(
            "Received control packet: {:#?}, remaining length={}",
            &self.packet_type, self.remaining_length
        );

        Ok(())
    }

    fn parse_variable_header(&mut self, buffer: &mut BytesMut) -> Result<(), Error> {
        println!("Parsing variable header, buffer len: {}", buffer.len());

        let protocol_name = parse_string(buffer).unwrap();

        let protocol_level = buffer[0];
        let connect_flags = buffer[1];
        buffer.advance(2);

        let keep_alive = parse_u16(buffer);

        println!("Parsed variable header");
        println!("\tprotocol name: {:#?}, protocol level: {:#08b}, connect flags: {:#08b}, keep alive: {}",
                 &protocol_name, &protocol_level, &connect_flags, &keep_alive);

        self.remaining_length -= self.variable_header_size() as u64;
        self.state = ParserState::Payload;

        Ok(())
    }

    fn parse_payload(&mut self, buffer: &mut BytesMut) -> Result<(), Error> {
        let client_id = parse_string(buffer).unwrap();
        self.remaining_length -= 2;

        println!("\tclient id: {:#?}", &client_id);
        self.remaining_length -= client_id.len() as u64;
        println!(
            "\tremaining_length: {:#?}, client id size: {:#?}",
            &self.remaining_length,
            &client_id.len()
        );

        Ok(())
    }

    fn variable_header_size(&self) -> usize {
        match &self.packet_type {
            ControlPacketType::Connect => 10,
            _ => 0,
        }
    }
}

impl Decoder for MqttCodec {
    fn parse_frame(&mut self, buffer: &mut BytesMut) -> Result<Option<ControlPacket>, Error> {
        println!("Parsing frame");
        loop {
            println!("Parsing frame, buffer len: {}", buffer.len());
            if buffer.is_empty() {
                return Ok(None);
            }

            return match &self.state {
                ParserState::FixedHeader => {
                    if buffer.len() >= FIXED_HEADER_MAX_SIZE {
                        self.parse_fixed_header(buffer)?;
                        continue;
                    }

                    Ok(None)
                }
                ParserState::VariableHeader => {
                    if buffer.len() >= self.variable_header_size() {
                        self.parse_variable_header(buffer)?;
                        continue;
                    }
                    Ok(None)
                }
                ParserState::Payload => {
                    if buffer.len() >= self.remaining_length as usize {
                        self.parse_payload(buffer)?;

                        if self.remaining_length == 0 {
                            return Ok(Some(self.packet.clone()));
                        } else {
                            continue;
                        }
                    }

                    Ok(None)
                }
            };
        }
    }
}

impl MqttCodec {
    fn create_packet(packet_type: &ControlPacketType) -> ControlPacket {
        match packet_type {
            Connect => ControlPacket::Connect(ConnectPacket::default()),
            _ => unimplemented!(),
        }
    }
}

impl Encoder for MqttCodec {
    fn encode(&mut self, packet: &ControlPacket, buffer: &mut BytesMut) -> Result<(), Error> {
        match packet {
            ControlPacket::ConnAck(ca) => {
                // ca.encode_fixed_header(buffer);
                // ca.encode_variable_header(buffer);
                // ca.encode_body(buffer);
            }
            _ => unimplemented!(),
        }
        Ok(())
    }
}
