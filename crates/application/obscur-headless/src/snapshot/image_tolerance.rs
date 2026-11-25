#[doc = include_str!("../../docs/api/snapshot/image_snapshot.doc/ImageTolerance.md")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ImageTolerance {
    /// The per-channel absolute delta at or below which two pixels are the same.
    max_channel_delta: u8,
    /// The fraction of pixels allowed to exceed `max_channel_delta`, in `0.0..=1.0`.
    max_differing_fraction: f64,
}

impl ImageTolerance {
    #[doc = include_str!("../../docs/api/snapshot/image_snapshot.doc/exact.md")]
    #[must_use]
    pub const fn exact() -> Self {
        Self {
            max_channel_delta: 0,
            max_differing_fraction: 0.0,
        }
    }

    #[doc = include_str!("../../docs/api/snapshot/image_snapshot.doc/new.md")]
    #[must_use]
    pub fn new(max_channel_delta: u8, max_differing_fraction: f64) -> Self {
        let max_differing_fraction = if max_differing_fraction.is_finite() {
            max_differing_fraction.clamp(0.0, 1.0)
        } else {
            0.0
        };
        Self {
            max_channel_delta,
            max_differing_fraction,
        }
    }

    /// The per-channel absolute delta at or below which two pixels are the same.
    #[must_use]
    pub const fn max_channel_delta(&self) -> u8 {
        self.max_channel_delta
    }

    /// The fraction of pixels allowed to exceed [`max_channel_delta`](Self::max_channel_delta).
    #[must_use]
    pub const fn max_differing_fraction(&self) -> f64 {
        self.max_differing_fraction
    }
}
