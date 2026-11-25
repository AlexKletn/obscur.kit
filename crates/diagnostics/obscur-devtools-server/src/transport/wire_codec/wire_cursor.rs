use super::WireError;

/// A bounds-checked reading cursor over a byte slice. Every read advances past
/// the bytes consumed and maps a short read to [`WireError::UnexpectedEof`].
pub(super) struct WireCursor<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> WireCursor<'a> {
    pub(super) const fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }

    fn take(&mut self, n: usize) -> Result<&'a [u8], WireError> {
        let end = self.pos.checked_add(n).ok_or(WireError::UnexpectedEof)?;
        let slice = self
            .data
            .get(self.pos..end)
            .ok_or(WireError::UnexpectedEof)?;
        self.pos = end;
        Ok(slice)
    }

    pub(super) fn read_u8(&mut self) -> Result<u8, WireError> {
        Ok(self.take(1)?[0])
    }

    pub(super) fn read_u16(&mut self) -> Result<u16, WireError> {
        let array = <[u8; 2]>::try_from(self.take(2)?).map_err(|_| WireError::UnexpectedEof)?;
        Ok(u16::from_le_bytes(array))
    }

    pub(super) fn read_u32(&mut self) -> Result<u32, WireError> {
        let array = <[u8; 4]>::try_from(self.take(4)?).map_err(|_| WireError::UnexpectedEof)?;
        Ok(u32::from_le_bytes(array))
    }

    pub(super) fn read_i32(&mut self) -> Result<i32, WireError> {
        let array = <[u8; 4]>::try_from(self.take(4)?).map_err(|_| WireError::UnexpectedEof)?;
        Ok(i32::from_le_bytes(array))
    }

    pub(super) fn read_u64(&mut self) -> Result<u64, WireError> {
        let array = <[u8; 8]>::try_from(self.take(8)?).map_err(|_| WireError::UnexpectedEof)?;
        Ok(u64::from_le_bytes(array))
    }

    pub(super) fn read_str(&mut self) -> Result<&'a str, WireError> {
        let len = self.read_u32()?;
        let n = usize::try_from(len).map_err(|_| WireError::UnexpectedEof)?;
        let bytes = self.take(n)?;
        core::str::from_utf8(bytes).map_err(|_| WireError::InvalidUtf8)
    }
}
