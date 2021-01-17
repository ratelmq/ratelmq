use bytes::{Buf, BufMut, BytesMut};
use std::str::Utf8Error;

pub fn parse_string(buffer: &mut BytesMut) -> Result<String, Utf8Error> {
    let string_size = parse_u16(buffer) as usize;

    let str_buf = buffer.split_to(string_size);
    let str = std::str::from_utf8(str_buf.as_ref())?;
    println!(
        "\tparsed string: {:#?}, of length: {:#?}",
        str, &string_size
    );

    Ok(String::from(str))
}

pub fn parse_u16(buffer: &mut BytesMut) -> u16 {
    let string_size: u16 = ((buffer[0] as u16) << 8) + buffer[1] as u16;
    buffer.advance(2);
    string_size
}

pub fn encode_remaining_length(mut remaining_length: u64, buffer: &mut BytesMut) {
    while remaining_length > 0 {
        let mut encoded_byte: u8 = (remaining_length % 128) as u8;
        remaining_length = remaining_length / 128;

        if remaining_length > 0 {
            encoded_byte = encoded_byte | 128;
        }

        buffer.put_u8(encoded_byte);
    }
}
