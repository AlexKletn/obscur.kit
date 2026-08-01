#![doc = include_str!("writer.doc/_module.md")]

use std::io::{self, Write};

#[doc = include_str!("writer.doc/write_message.md")]
pub fn write_message<W: Write>(writer: &mut W, body: &str) -> io::Result<()> {
    write!(writer, "Content-Length: {}\r\n\r\n", body.len())?;
    writer.write_all(body.as_bytes())?;
    writer.flush()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transport::read_message;
    use std::io::Cursor;

    #[test]
    fn frames_body_with_byte_length_header() {
        let mut out = Vec::new();
        write_message(&mut out, "{\"ok\":true}").unwrap();
        assert_eq!(out, b"Content-Length: 11\r\n\r\n{\"ok\":true}");
    }

    #[test]
    fn writer_and_reader_round_trip() {
        let body = "{\"method\":\"initialize\",\"params\":{\"x\":\"café\"}}";
        let mut out = Vec::new();
        write_message(&mut out, body).unwrap();
        let mut cursor = Cursor::new(out);
        assert_eq!(read_message(&mut cursor).unwrap().as_deref(), Some(body));
    }
}
