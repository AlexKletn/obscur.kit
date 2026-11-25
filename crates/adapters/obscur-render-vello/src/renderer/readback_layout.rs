/// RGBA row sizes in bytes, with the GPU copy alignment applied to the second.
pub(super) fn readback_row_bytes(width: u32) -> Option<(u32, u32)> {
    let unpadded = width.checked_mul(4)?;
    let padded = unpadded.checked_next_multiple_of(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT)?;
    Some((unpadded, padded))
}

#[cfg(test)]
mod tests {
    use super::readback_row_bytes;

    #[test]
    fn checks_both_row_size_and_alignment_overflow() {
        assert_eq!(readback_row_bytes(65), Some((260, 512)));
        assert_eq!(readback_row_bytes(u32::MAX), None);
        assert_eq!(readback_row_bytes(u32::MAX / 4), None);
    }
}
