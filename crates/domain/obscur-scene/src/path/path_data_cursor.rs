//! Reads the byte-oriented operands of SVG path commands.

use super::{PathDataError, PathDataFault};

pub(super) struct PathDataCursor<'a> {
    bytes: &'a [u8],
    at: usize,
}

impl<'a> PathDataCursor<'a> {
    pub(super) const fn new(data: &'a str) -> Self {
        Self {
            bytes: data.as_bytes(),
            at: 0,
        }
    }

    pub(super) fn peek(&self) -> Option<u8> {
        self.bytes.get(self.at).copied()
    }

    pub(super) const fn offset(&self) -> usize {
        self.at
    }

    pub(super) fn advance(&mut self) {
        self.at += 1;
    }

    /// Reads one SVG arc flag, leaving any adjacent flag or coordinate for the next operand.
    pub(super) fn flag(&mut self) -> Result<bool, PathDataError> {
        self.skip_separators();
        match self.bytes.get(self.at) {
            Some(&byte @ (b'0' | b'1')) => {
                self.at += 1;
                Ok(byte == b'1')
            }
            _ => Err(self.fault(PathDataFault::InvalidArcFlag)),
        }
    }

    pub(super) fn number(&mut self) -> Result<f32, PathDataError> {
        self.skip_separators();
        let start = self.at;
        self.skip_sign();
        self.skip_digits();
        if self.peek() == Some(b'.') {
            self.advance();
            self.skip_digits();
        }
        if matches!(self.peek(), Some(b'e' | b'E')) {
            let exponent = self.offset();
            self.advance();
            self.skip_sign();
            let digits = self.offset();
            self.skip_digits();
            if self.offset() == digits {
                // Leave a bare exponent marker for command parsing to report.
                self.at = exponent;
            }
        }
        core::str::from_utf8(&self.bytes[start..self.at])
            .ok()
            .and_then(|text| text.parse::<f32>().ok())
            .filter(|value| value.is_finite())
            .ok_or(PathDataError {
                at: start,
                what: PathDataFault::MissingNumber,
            })
    }

    fn skip_sign(&mut self) {
        if matches!(self.peek(), Some(b'+' | b'-')) {
            self.advance();
        }
    }

    fn skip_digits(&mut self) {
        while self.peek().is_some_and(|byte| byte.is_ascii_digit()) {
            self.advance();
        }
    }

    pub(super) fn skip_separators(&mut self) {
        while matches!(self.peek(), Some(b' ' | b',' | b'\t' | b'\n' | b'\r')) {
            self.advance();
        }
    }

    pub(super) fn fault(&self, what: PathDataFault) -> PathDataError {
        PathDataError { at: self.at, what }
    }
}
