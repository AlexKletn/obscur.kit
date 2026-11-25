#[doc = include_str!("../../docs/api/snapshot/image_snapshot.doc/ImageDiff.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ImageDiff {
    /// Pixels whose worst channel exceeded the tolerance's noise floor.
    differing_pixels: u64,
    /// Pixels compared.
    total_pixels: u64,
    /// The largest per-channel delta seen anywhere, tolerated or not.
    max_channel_delta: u8,
}

impl ImageDiff {
    pub(super) const fn new(
        differing_pixels: u64,
        total_pixels: u64,
        max_channel_delta: u8,
    ) -> Self {
        Self {
            differing_pixels,
            total_pixels,
            max_channel_delta,
        }
    }

    /// Pixels whose worst channel exceeded the tolerance's noise floor.
    #[must_use]
    pub const fn differing_pixels(&self) -> u64 {
        self.differing_pixels
    }

    /// Pixels compared.
    #[must_use]
    pub const fn total_pixels(&self) -> u64 {
        self.total_pixels
    }

    #[doc = include_str!("../../docs/api/snapshot/image_snapshot.doc/max_channel_delta_4.md")]
    #[must_use]
    pub const fn max_channel_delta(&self) -> u8 {
        self.max_channel_delta
    }

    #[doc = include_str!("../../docs/api/snapshot/image_snapshot.doc/differing_fraction.md")]
    #[must_use]
    pub fn differing_fraction(&self) -> f64 {
        if self.total_pixels == 0 {
            return 0.0;
        }
        // Both counts are pixel counts of a raster that fits in memory, so the
        // f64 mantissa is not a concern here.
        self.differing_pixels as f64 / self.total_pixels as f64
    }
}
