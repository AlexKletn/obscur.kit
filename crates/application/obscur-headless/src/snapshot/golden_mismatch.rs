use super::ImageDiff;

#[doc = include_str!("../../docs/api/snapshot/image_snapshot.doc/GoldenMismatch.md")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GoldenMismatch {
    /// The two rasters are different sizes, so no per-pixel comparison exists.
    DimensionMismatch {
        /// The expected raster's dimensions.
        expected: (u32, u32),
        /// The actual raster's dimensions.
        actual: (u32, u32),
    },
    /// One side carries no pixels — it was built by
    /// [`from_parts`](ImageSnapshot::from_parts) from a bare hash, which a
    /// tolerance cannot be applied to.
    PixelsUnavailable,
    /// The two rasters agree on dimensions but carry different byte counts, so
    /// they do not share a pixel layout.
    ByteLengthMismatch {
        /// The expected raster's byte count.
        expected: usize,
        /// The actual raster's byte count.
        actual: usize,
    },
    /// More pixels exceeded the noise floor than the tolerance allows.
    TooManyDifferingPixels {
        /// What was measured.
        diff: ImageDiff,
        /// The fraction the tolerance permitted.
        allowed: f64,
    },
}

impl std::fmt::Display for GoldenMismatch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DimensionMismatch { expected, actual } => write!(
                f,
                "raster dimensions differ: expected {}x{}, got {}x{}",
                expected.0, expected.1, actual.0, actual.1
            ),
            Self::PixelsUnavailable => write!(
                f,
                "a tolerant comparison needs pixels on both sides; one snapshot carries only a hash"
            ),
            Self::ByteLengthMismatch { expected, actual } => write!(
                f,
                "raster byte counts differ: expected {expected}, got {actual}"
            ),
            Self::TooManyDifferingPixels { diff, allowed } => write!(
                f,
                "{} of {} pixels ({:.4}) exceeded the channel noise floor, tolerance allows {:.4}; worst channel delta {}",
                diff.differing_pixels(),
                diff.total_pixels(),
                diff.differing_fraction(),
                allowed,
                diff.max_channel_delta(),
            ),
        }
    }
}

impl std::error::Error for GoldenMismatch {}
