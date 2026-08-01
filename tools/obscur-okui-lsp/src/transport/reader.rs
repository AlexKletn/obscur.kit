#![doc = include_str!("reader.doc/_module.md")]

use std::io::{self, BufRead};

#[doc = include_str!("reader.doc/read_message.md")]
pub fn read_message<R: BufRead>(reader: &mut R) -> io::Result<Option<String>> {
    let mut content_length: Option<usize> = None;
    let mut saw_header = false;

    loop {
        let mut line = Vec::new();
        let read = reader.read_until(b'\n', &mut line)?;
        if read == 0 {
            // End of stream. Clean only if it happened before any header bytes.
            if saw_header {
                return Err(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "stream ended inside message headers",
                ));
            }
            return Ok(None);
        }
        // Strip the trailing CR/LF.
        while matches!(line.last(), Some(b'\n' | b'\r')) {
            line.pop();
        }
        if line.is_empty() {
            break; // blank line: end of headers
        }
        saw_header = true;
        let header = String::from_utf8_lossy(&line);
        if let Some((name, value)) = header.split_once(':')
            && name.trim().eq_ignore_ascii_case("content-length")
        {
            content_length = value.trim().parse::<usize>().ok();
        }
    }

    let length = content_length.ok_or_else(|| {
        io::Error::new(io::ErrorKind::InvalidData, "missing Content-Length header")
    })?;
    let mut body = vec![0u8; length];
    reader.read_exact(&mut body)?;
    String::from_utf8(body)
        .map(Some)
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn reads_a_single_framed_message() {
        let body = "{\"jsonrpc\":\"2.0\"}";
        let raw = format!("Content-Length: {}\r\n\r\n{body}", body.len());
        let mut cursor = Cursor::new(raw.into_bytes());
        assert_eq!(read_message(&mut cursor).unwrap(), Some(body.to_owned()));
        // Second read hits clean EOF.
        assert_eq!(read_message(&mut cursor).unwrap(), None);
    }

    #[test]
    fn reads_back_to_back_messages_and_ignores_extra_headers() {
        let first = "{\"a\":1}";
        let second = "{\"b\":2}";
        let raw = format!(
            "Content-Type: application/vscode-jsonrpc; charset=utf-8\r\nContent-Length: {}\r\n\r\n{first}Content-Length: {}\r\n\r\n{second}",
            first.len(),
            second.len(),
        );
        let mut cursor = Cursor::new(raw.into_bytes());
        assert_eq!(read_message(&mut cursor).unwrap().as_deref(), Some(first));
        assert_eq!(read_message(&mut cursor).unwrap().as_deref(), Some(second));
        assert_eq!(read_message(&mut cursor).unwrap(), None);
    }

    #[test]
    fn honours_content_length_in_bytes_for_multibyte_utf8() {
        let body = "{\"s\":\"café\"}"; // 'é' is two UTF-8 bytes
        let raw = format!("Content-Length: {}\r\n\r\n{body}", body.len());
        let mut cursor = Cursor::new(raw.into_bytes());
        assert_eq!(read_message(&mut cursor).unwrap().as_deref(), Some(body));
    }

    #[test]
    fn errors_when_headers_are_truncated() {
        let mut cursor = Cursor::new(b"Content-Length: 5\r\n".to_vec());
        assert!(read_message(&mut cursor).is_err());
    }
}
