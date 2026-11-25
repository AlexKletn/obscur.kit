use super::UiIrDecodeError;

/// A bounds-checked reading cursor over a byte slice. Every read advances
/// [`pos`](DecodeCursor::pos) and maps a short read to
/// [`UiIrDecodeError::UnexpectedEof`].
pub(super) struct DecodeCursor<'a> {
    data: &'a [u8],
    pos: usize,
    remaining_nodes: Option<usize>,
}

impl<'a> DecodeCursor<'a> {
    /// Creates a cursor positioned at the start of `data`.
    pub(super) const fn new(data: &'a [u8], max_nodes: Option<usize>) -> Self {
        Self {
            data,
            pos: 0,
            remaining_nodes: max_nodes,
        }
    }

    pub(super) fn check_depth(&self, depth: usize) -> Result<(), UiIrDecodeError> {
        if self.remaining_nodes.is_some() && depth >= 64 {
            Err(UiIrDecodeError::LimitExceeded)
        } else {
            Ok(())
        }
    }

    pub(super) fn consume_node(&mut self, depth: usize) -> Result<(), UiIrDecodeError> {
        self.check_depth(depth)?;
        if let Some(remaining) = &mut self.remaining_nodes {
            *remaining = remaining
                .checked_sub(1)
                .ok_or(UiIrDecodeError::LimitExceeded)?;
        }
        Ok(())
    }

    pub(super) fn is_at_end(&self) -> bool {
        self.pos == self.data.len()
    }

    /// Returns the next `n` bytes, advancing past them.
    pub(super) fn take(&mut self, n: usize) -> Result<&'a [u8], UiIrDecodeError> {
        let end = self
            .pos
            .checked_add(n)
            .ok_or(UiIrDecodeError::UnexpectedEof)?;
        let slice = self
            .data
            .get(self.pos..end)
            .ok_or(UiIrDecodeError::UnexpectedEof)?;
        self.pos = end;
        Ok(slice)
    }

    /// Reads a single byte.
    pub(super) fn read_u8(&mut self) -> Result<u8, UiIrDecodeError> {
        let bytes = self.take(1)?;
        Ok(bytes[0])
    }

    /// Reads a little-endian `u16`.
    pub(super) fn read_u16(&mut self) -> Result<u16, UiIrDecodeError> {
        let bytes = self.take(2)?;
        let array = <[u8; 2]>::try_from(bytes).map_err(|_| UiIrDecodeError::UnexpectedEof)?;
        Ok(u16::from_le_bytes(array))
    }

    /// Reads a little-endian `u32`.
    pub(super) fn read_u32(&mut self) -> Result<u32, UiIrDecodeError> {
        let bytes = self.take(4)?;
        let array = <[u8; 4]>::try_from(bytes).map_err(|_| UiIrDecodeError::UnexpectedEof)?;
        Ok(u32::from_le_bytes(array))
    }

    /// Reads a little-endian `u64`.
    pub(super) fn read_u64(&mut self) -> Result<u64, UiIrDecodeError> {
        let bytes = self.take(8)?;
        let array = <[u8; 8]>::try_from(bytes).map_err(|_| UiIrDecodeError::UnexpectedEof)?;
        Ok(u64::from_le_bytes(array))
    }

    /// Reads a little-endian `i64`.
    pub(super) fn read_i64(&mut self) -> Result<i64, UiIrDecodeError> {
        let bytes = self.take(8)?;
        let array = <[u8; 8]>::try_from(bytes).map_err(|_| UiIrDecodeError::UnexpectedEof)?;
        Ok(i64::from_le_bytes(array))
    }

    /// Reads a little-endian `f64`.
    pub(super) fn read_f64(&mut self) -> Result<f64, UiIrDecodeError> {
        let bytes = self.take(8)?;
        let array = <[u8; 8]>::try_from(bytes).map_err(|_| UiIrDecodeError::UnexpectedEof)?;
        Ok(f64::from_le_bytes(array))
    }

    /// Reads a `u32`-length-prefixed UTF-8 string.
    pub(super) fn read_str(&mut self) -> Result<&'a str, UiIrDecodeError> {
        let len = self.read_u32()?;
        let n = usize::try_from(len).map_err(|_| UiIrDecodeError::UnexpectedEof)?;
        let bytes = self.take(n)?;
        std::str::from_utf8(bytes).map_err(|_| UiIrDecodeError::InvalidUtf8)
    }
}
