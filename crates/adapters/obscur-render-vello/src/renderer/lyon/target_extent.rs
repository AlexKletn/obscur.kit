/// Rounds to a 32-pixel allocation boundary within the backing surface.
pub(super) fn rounded_target_extent(pixels: u32, limit: u32) -> u32 {
    pixels
        .checked_next_multiple_of(32)
        .map_or(limit, |rounded| rounded.min(limit))
}

#[cfg(test)]
mod tests {
    use super::rounded_target_extent;

    #[test]
    fn rounding_respects_the_surface_limit_even_at_integer_boundaries() {
        assert_eq!(rounded_target_extent(33, 100), 64);
        assert_eq!(rounded_target_extent(33, 50), 50);
        assert_eq!(rounded_target_extent(u32::MAX, 100), 100);
    }
}
