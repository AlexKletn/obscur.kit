//! A backend-neutral capture of a rendered raster image: [`ImageSnapshot`],
//! and the §7.14 tolerance it is compared under: [`ImageTolerance`].

use std::sync::Arc;

use super::{GoldenMismatch, ImageDiff, ImageTolerance};

#[doc = include_str!("../../docs/api/snapshot/image_snapshot.doc/ImageSnapshot.md")]
#[derive(Debug, Clone)]
pub struct ImageSnapshot {
    /// The image width in physical pixels.
    width: u32,
    /// The image height in physical pixels.
    height: u32,
    /// A stable FNV-1a content hash of the image's raw bytes.
    pixel_hash: u64,
    /// The raster itself, when the snapshot was built from one.
    pixels: Option<Arc<[u8]>>,
}

impl ImageSnapshot {
    /// The FNV-1a 64-bit offset basis.
    const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
    /// The FNV-1a 64-bit prime.
    const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

    #[doc = include_str!("../../docs/api/snapshot/image_snapshot.doc/from_rgba.md")]
    #[must_use]
    pub fn from_rgba(width: u32, height: u32, bytes: &[u8]) -> Self {
        let mut hash = Self::FNV_OFFSET;
        for &byte in bytes {
            hash ^= u64::from(byte);
            hash = hash.wrapping_mul(Self::FNV_PRIME);
        }
        Self {
            width,
            height,
            pixel_hash: hash,
            pixels: Some(Arc::from(bytes)),
        }
    }

    #[doc = include_str!("../../docs/api/snapshot/image_snapshot.doc/from_parts.md")]
    #[must_use]
    pub const fn from_parts(width: u32, height: u32, pixel_hash: u64) -> Self {
        Self {
            width,
            height,
            pixel_hash,
            pixels: None,
        }
    }

    /// The image width in physical pixels.
    #[must_use]
    pub const fn width(&self) -> u32 {
        self.width
    }

    /// The image height in physical pixels.
    #[must_use]
    pub const fn height(&self) -> u32 {
        self.height
    }

    /// The stable content hash of the image's bytes.
    #[must_use]
    pub const fn pixel_hash(&self) -> u64 {
        self.pixel_hash
    }

    /// The captured raster, if this snapshot was built from one.
    #[must_use]
    pub fn pixels(&self) -> Option<&[u8]> {
        self.pixels.as_deref()
    }

    /// Whether the image has zero area.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.width == 0 || self.height == 0
    }

    #[doc = include_str!("../../docs/api/snapshot/image_snapshot.doc/compare_within.md")]
    pub fn compare_within(
        &self,
        other: &Self,
        tolerance: ImageTolerance,
    ) -> Result<ImageDiff, GoldenMismatch> {
        if self.width != other.width || self.height != other.height {
            return Err(GoldenMismatch::DimensionMismatch {
                expected: (self.width, self.height),
                actual: (other.width, other.height),
            });
        }

        let (Some(golden), Some(actual)) = (self.pixels(), other.pixels()) else {
            return Err(GoldenMismatch::PixelsUnavailable);
        };

        if golden.len() != actual.len() {
            return Err(GoldenMismatch::ByteLengthMismatch {
                expected: golden.len(),
                actual: actual.len(),
            });
        }

        let mut differing_pixels: u64 = 0;
        let mut worst_delta: u8 = 0;
        for (lhs, rhs) in golden.chunks_exact(4).zip(actual.chunks_exact(4)) {
            let mut pixel_delta: u8 = 0;
            for (a, b) in lhs.iter().zip(rhs.iter()) {
                pixel_delta = pixel_delta.max(a.abs_diff(*b));
            }
            worst_delta = worst_delta.max(pixel_delta);
            if pixel_delta > tolerance.max_channel_delta() {
                differing_pixels += 1;
            }
        }

        let diff = ImageDiff::new(differing_pixels, (golden.len() / 4) as u64, worst_delta);

        if diff.differing_fraction() > tolerance.max_differing_fraction() {
            return Err(GoldenMismatch::TooManyDifferingPixels {
                diff,
                allowed: tolerance.max_differing_fraction(),
            });
        }
        Ok(diff)
    }
}

#[doc = include_str!("../../docs/api/snapshot/image_snapshot.doc/PartialEq.md")]
impl PartialEq for ImageSnapshot {
    fn eq(&self, other: &Self) -> bool {
        self.width == other.width
            && self.height == other.height
            && self.pixel_hash == other.pixel_hash
    }
}

impl Eq for ImageSnapshot {}

impl std::hash::Hash for ImageSnapshot {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // Hashes exactly the fields `PartialEq` compares, as the `Hash`/`Eq`
        // contract requires.
        self.width.hash(state);
        self.height.hash(state);
        self.pixel_hash.hash(state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A 2×2 opaque-white raster.
    fn white_2x2() -> ImageSnapshot {
        ImageSnapshot::from_rgba(2, 2, &[0xff; 16])
    }

    #[test]
    fn same_bytes_hash_equal() {
        let a = ImageSnapshot::from_rgba(2, 2, &[1, 2, 3, 4, 5, 6, 7, 8]);
        let b = ImageSnapshot::from_rgba(2, 2, &[1, 2, 3, 4, 5, 6, 7, 8]);
        assert_eq!(a, b);
        assert_eq!(a.width(), 2);
        assert_eq!(a.height(), 2);
        assert!(!a.is_empty());
    }

    #[test]
    fn different_bytes_hash_differently() {
        let a = ImageSnapshot::from_rgba(2, 2, &[1, 2, 3, 4]);
        let b = ImageSnapshot::from_rgba(2, 2, &[1, 2, 3, 5]);
        assert_ne!(a.pixel_hash(), b.pixel_hash());
    }

    #[test]
    fn empty_dimensions_report_empty() {
        assert!(ImageSnapshot::from_rgba(0, 4, &[]).is_empty());
        assert!(ImageSnapshot::from_rgba(4, 0, &[]).is_empty());
        assert!(!ImageSnapshot::from_rgba(1, 1, &[0, 0, 0, 0]).is_empty());
    }

    #[test]
    fn from_parts_round_trips_fields() {
        let img = ImageSnapshot::from_parts(16, 9, 0xdead_beef);
        assert_eq!(img.width(), 16);
        assert_eq!(img.height(), 9);
        assert_eq!(img.pixel_hash(), 0xdead_beef);
    }

    #[test]
    fn from_rgba_retains_the_raster_and_from_parts_does_not() {
        assert_eq!(white_2x2().pixels(), Some(&[0xffu8; 16][..]));
        assert!(ImageSnapshot::from_parts(2, 2, 7).pixels().is_none());
    }

    #[test]
    fn a_hash_only_snapshot_reports_that_it_cannot_be_compared_rather_than_passing() {
        // The load-bearing half: a golden check that cannot run must not be a
        // golden check that passed.
        let hashed = ImageSnapshot::from_parts(2, 2, white_2x2().pixel_hash());
        assert_eq!(
            hashed.compare_within(&white_2x2(), ImageTolerance::new(8, 0.05)),
            Err(GoldenMismatch::PixelsUnavailable)
        );
        // ...even though exact equality is perfectly happy with it.
        assert_eq!(hashed, white_2x2());
    }

    #[test]
    fn dimension_and_byte_length_mismatches_are_distinguished() {
        let wide = ImageSnapshot::from_rgba(4, 1, &[0xff; 16]);
        assert_eq!(
            white_2x2().compare_within(&wide, ImageTolerance::new(255, 1.0)),
            Err(GoldenMismatch::DimensionMismatch {
                expected: (2, 2),
                actual: (4, 1),
            })
        );

        // Same declared dimensions, inconsistent buffers: no shared layout.
        let short = ImageSnapshot::from_rgba(2, 2, &[0xff; 8]);
        assert_eq!(
            white_2x2().compare_within(&short, ImageTolerance::new(255, 1.0)),
            Err(GoldenMismatch::ByteLengthMismatch {
                expected: 16,
                actual: 8,
            })
        );
    }

    #[test]
    fn a_delta_at_the_noise_floor_is_inside_it_and_one_past_it_is_not() {
        // The boundary is `<=`: `max_channel_delta` is the largest tolerated
        // delta, not the smallest rejected one.
        let golden = white_2x2();
        let off_by_four = ImageSnapshot::from_rgba(2, 2, &[0xfb; 16]);
        let diff = golden
            .compare_within(&off_by_four, ImageTolerance::new(4, 0.0))
            .expect("a delta of exactly 4 is within a noise floor of 4");
        assert_eq!(diff.differing_pixels(), 0);
        assert_eq!(diff.max_channel_delta(), 4, "measured, but tolerated");

        let off_by_five = ImageSnapshot::from_rgba(2, 2, &[0xfa; 16]);
        assert!(matches!(
            golden.compare_within(&off_by_five, ImageTolerance::new(4, 0.0)),
            Err(GoldenMismatch::TooManyDifferingPixels { .. })
        ));
    }

    #[test]
    fn the_diff_reports_headroom_on_success() {
        // One of four pixels moved past the floor; a 50% cap accepts it and
        // still reports the 25% that moved.
        let golden = white_2x2();
        let mut bytes = [0xffu8; 16];
        bytes[0] = 0x00;
        let actual = ImageSnapshot::from_rgba(2, 2, &bytes);

        let diff = golden
            .compare_within(&actual, ImageTolerance::new(4, 0.5))
            .expect("25% differing is inside a 50% cap");
        assert_eq!(diff.differing_pixels(), 1);
        assert_eq!(diff.total_pixels(), 4);
        assert!((diff.differing_fraction() - 0.25).abs() < f64::EPSILON);
        assert_eq!(diff.max_channel_delta(), 255);
    }

    #[test]
    fn exact_tolerance_permits_nothing() {
        let golden = white_2x2();
        let mut bytes = [0xffu8; 16];
        bytes[0] = 0xfe;
        let one_bit_off = ImageSnapshot::from_rgba(2, 2, &bytes);

        assert!(
            golden
                .compare_within(&golden.clone(), ImageTolerance::exact())
                .is_ok()
        );
        assert!(matches!(
            golden.compare_within(&one_bit_off, ImageTolerance::exact()),
            Err(GoldenMismatch::TooManyDifferingPixels { .. })
        ));
    }

    #[test]
    fn a_non_finite_fraction_clamps_shut_rather_than_accepting_everything() {
        // `NaN > NaN` is false, so a propagated NaN would make every comparison
        // pass. It must clamp to the *strict* end, not the permissive one.
        let nan = ImageTolerance::new(0, f64::NAN);
        assert!((nan.max_differing_fraction() - 0.0).abs() < f64::EPSILON);

        let mut bytes = [0xffu8; 16];
        bytes[0] = 0x00;
        assert!(matches!(
            white_2x2().compare_within(&ImageSnapshot::from_rgba(2, 2, &bytes), nan),
            Err(GoldenMismatch::TooManyDifferingPixels { .. })
        ));

        assert!((ImageTolerance::new(0, 2.0).max_differing_fraction() - 1.0).abs() < f64::EPSILON);
        assert!((ImageTolerance::new(0, -1.0).max_differing_fraction() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn equality_ignores_retained_pixels_so_hash_only_snapshots_still_compare() {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let raster = white_2x2();
        let hashed = ImageSnapshot::from_parts(2, 2, raster.pixel_hash());
        assert_eq!(raster, hashed);

        // `Hash` must agree with `Eq`.
        let digest = |img: &ImageSnapshot| {
            let mut hasher = DefaultHasher::new();
            img.hash(&mut hasher);
            hasher.finish()
        };
        assert_eq!(digest(&raster), digest(&hashed));
    }
}
