/// Converts Wayland presentation time to microseconds, truncating sub-microsecond
/// precision and saturating timestamps beyond the representable range.
#[must_use]
pub const fn presentation_timestamp_micros(
    seconds_high: u32,
    seconds_low: u32,
    nanoseconds: u32,
) -> u64 {
    let seconds = ((seconds_high as u64) << 32) | seconds_low as u64;
    seconds
        .saturating_mul(1_000_000)
        .saturating_add(nanoseconds as u64 / 1_000)
}

#[cfg(test)]
mod tests {
    use super::presentation_timestamp_micros;

    #[test]
    fn combines_both_seconds_words_and_truncates_nanoseconds() {
        assert_eq!(
            presentation_timestamp_micros(1, 2, 3_999),
            4_294_967_298_000_003
        );
    }

    #[test]
    fn saturates_timestamps_beyond_u64_microseconds() {
        assert_eq!(
            presentation_timestamp_micros(u32::MAX, u32::MAX, 0),
            u64::MAX
        );
        let seconds = u64::MAX / 1_000_000;
        assert_eq!(
            presentation_timestamp_micros((seconds >> 32) as u32, seconds as u32, 999_999_999),
            u64::MAX
        );
    }
}
