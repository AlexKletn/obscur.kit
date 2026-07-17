//! A backend-neutral capture of a rendered raster image: [`ImageSnapshot`],
//! and the §7.14 tolerance it is compared under: [`ImageTolerance`].

use std::sync::Arc;

/// How much variation a raster golden comparison tolerates (spec §7.14).
///
/// # What the spec asks for
///
/// §7.14 is one sentence: *"Raster golden tests are allowed with explicit
/// tolerances for anti-aliasing and font/backend variation."* §5.2 adds
/// *"Golden tests use tolerances"* under its **allowed differences** list —
/// rasterization, anti-aliasing, font rendering details, backend-specific
/// precision.
///
/// Neither sentence prescribes a *metric*. What they pin down is that the
/// tolerance must be **explicit** and must absorb the named **sources** of
/// variation. This type therefore derives its shape from those sources rather
/// than from a number the spec never gives:
///
/// - **backend-specific precision** — a different blend or gamma rounding
///   shifts *every* pixel by a *small* amount. Absorbed by
///   [`max_channel_delta`](Self::max_channel_delta): a per-channel noise floor.
/// - **anti-aliasing and font rendering** — two rasterizers disagree only where
///   coverage is *partial*: glyph and shape edges. That is a *small fraction* of
///   pixels, but on those pixels the delta is *unbounded* — at a hard black-on-
///   white edge, 49% vs 51% coverage is a full swing. Absorbed by
///   [`max_differing_fraction`](Self::max_differing_fraction): a cap on *how
///   many* pixels may exceed the noise floor, with no cap on by how much.
///
/// # Why two knobs and not one
///
/// The two sources are opposite in shape — many pixels × small delta versus few
/// pixels × large delta — so no single scalar grants both permissions. Each of
/// the one-knob designs was measured against real renders in
/// `obscur-render-cpu`'s `golden_tolerance.rs`, and each fails, in *opposite*
/// directions:
///
/// - **A lone noise floor is too loose.** Passing an AA edge needs a floor of
///   240, and a floor of 240 also passes an 8/255 recolouring of the entire
///   fill — a real regression waved straight through.
/// - **A lone differing-pixel cap is too strict.** With no floor, the 4/255
///   rounding difference §5.2 explicitly permits moves 52% of the image into
///   "differing", so no cap that still rejects regressions can accept it.
///   "Differs at all" cannot tell a rounding wobble from a wrong colour.
///
/// Together each knob grants exactly one of the two permissions, and a
/// difference that is neither shape is rejected.
///
/// # A deliberate omission
///
/// There is **no cap on the channel delta of a differing pixel**, and that is
/// not an oversight: legitimate AA variation on a partial-coverage pixel really
/// can span the full channel range, so such a cap would reject the very
/// difference §5.2 permits.
///
/// # What this therefore does not catch — read before trusting it
///
/// A regression with a **small footprint** passes. The two knobs bound *how far*
/// a pixel may move and *how many* may move, and nothing bounds where: a 10×10
/// element dropped from a 64×64 surface moves 100 pixels — 2.44% — at a full
/// 255 delta, and is accepted by any fraction above that, though §5.2 puts scene
/// geometry under *must match*. This was measured, not reasoned about.
///
/// The earlier version of this doc claimed the opposite ("a real regression moves
/// more pixels than the tolerated fraction"). It does not follow, and it is the
/// worse error: a reader trusting it concludes they are guarded where they are
/// not. The fraction cap protects against a *diffuse* regression, and only that.
///
/// Closing it needs a discriminator neither knob has, because the honest
/// difference between AA noise and a dropped element is **shape**, not magnitude
/// or count: AA differences hug an outline and are a pixel or two wide, while a
/// dropped element is a contiguous filled region. A per-tile fraction, or a
/// reject-if-any-NxN-block-differs rule, would express that — both need a size
/// constant this workspace has no evidence to pick yet, so neither is guessed at
/// here.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ImageTolerance {
    /// The per-channel absolute delta at or below which two pixels are the same.
    max_channel_delta: u8,
    /// The fraction of pixels allowed to exceed `max_channel_delta`, in `0.0..=1.0`.
    max_differing_fraction: f64,
}

impl ImageTolerance {
    /// A tolerance that permits nothing: byte-for-byte equality.
    ///
    /// This is the honest default for a *same-backend determinism* check (the
    /// same rasterizer, twice, on the same scene), which is not what §7.14's
    /// tolerance is for — see [`ImageSnapshot::compare_within`].
    #[must_use]
    pub const fn exact() -> Self {
        Self {
            max_channel_delta: 0,
            max_differing_fraction: 0.0,
        }
    }

    /// An explicit tolerance.
    ///
    /// `max_channel_delta` is the per-channel noise floor; `max_differing_fraction`
    /// is the share of pixels allowed to exceed it, clamped into `0.0..=1.0`.
    ///
    /// A non-finite `max_differing_fraction` is clamped to `0.0`, not propagated:
    /// `NaN` compares false against every threshold, so letting it through would
    /// produce a golden check that silently accepts *everything*.
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

/// What a tolerant comparison measured (spec §7.14).
///
/// Returned on success so a passing golden test can still report its *residual*
/// — the headroom left before the tolerance would have rejected it.
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

    /// The largest per-channel delta seen anywhere, tolerated or not.
    ///
    /// This counts *every* pixel, including ones inside the noise floor, so it
    /// is a diagnostic rather than a verdict.
    #[must_use]
    pub const fn max_channel_delta(&self) -> u8 {
        self.max_channel_delta
    }

    /// The share of compared pixels that exceeded the noise floor.
    ///
    /// An empty image has no differing pixels and reports `0.0`.
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

/// Every way a raster golden comparison fails to hold (spec §7.14).
///
/// This deliberately folds *"compared, and differs too much"* together with
/// *"could not compare at all"*. A golden check that cannot run is not a golden
/// check that passed: if `PixelsUnavailable` were reported as success, a
/// snapshot rebuilt from a bare hash would wave every regression through.
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

/// A deterministic, backend-neutral capture of a rendered raster image
/// (spec §7.4).
///
/// The rendered image is the one §7.4 stage that is *backend-dependent*: pixels
/// come out of a concrete rasterizer, not the backend-independent pipeline. This
/// type keeps `obscur-headless` free of any GPU or rasterizer dependency by
/// never *producing* pixels — [`from_rgba`](Self::from_rgba) takes whatever
/// raster a backend already made (for example the software `obscur-render-cpu`
/// backend's `tiny_skia::Pixmap::data`) and both hashes **and retains** it.
///
/// # Two comparisons, for two different questions
///
/// - [`PartialEq`] is **exact**: it compares dimensions and the FNV-1a content
///   hash. This answers *"did the same backend produce the same bytes twice?"* —
///   a determinism check, where any difference is a defect.
/// - [`compare_within`](Self::compare_within) is **tolerant** (spec §7.14). This
///   answers *"does this render still match its golden, allowing for the
///   variation §5.2 permits?"* — anti-aliasing, font rendering, backend
///   precision.
///
/// The two are kept separate on purpose. A hash cannot express a tolerance (one
/// bit differs and the digest is unrelated), so the hash is retained for the
/// question it does answer, and the pixels are retained for the one it does not.
///
/// # Cost
///
/// [`from_rgba`](Self::from_rgba) retains the raster, so a snapshot is as large
/// as the image it captured. The buffer is shared behind an [`Arc`], so cloning
/// a snapshot — as [`SnapshotNormalizer`](super::SnapshotNormalizer) does — does
/// not copy it. [`from_parts`](Self::from_parts) builds a hash-only snapshot for
/// callers that want the stage identity without the bytes; such a snapshot
/// **cannot** take part in a tolerant comparison, and says so rather than
/// passing.
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

    /// Captures an image snapshot from raw pixel `bytes` of a `width` × `height`
    /// raster.
    ///
    /// Any 4-bytes-per-pixel layout works — RGBA8, BGRA8, premultiplied or not —
    /// as long as both sides of a comparison use the same one, since a
    /// per-channel delta is computed bytewise.
    ///
    /// The `bytes` are hashed with FNV-1a into a stable content digest and
    /// retained for [`compare_within`](Self::compare_within): identical
    /// dimensions and bytes always produce an equal snapshot.
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

    /// Builds a **hash-only** snapshot from an already-computed content hash,
    /// for callers that hashed the pixels themselves.
    ///
    /// The result carries no raster, so [`compare_within`](Self::compare_within)
    /// on it reports [`GoldenMismatch::PixelsUnavailable`] rather than a verdict.
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

    /// Compares this render against a golden `other` under an **explicit
    /// tolerance** (spec §7.14).
    ///
    /// `self` is the golden and `other` the render under test; the comparison is
    /// symmetric, so the roles only shape the error's `expected`/`actual`.
    ///
    /// Returns the measured [`ImageDiff`] when the render is within tolerance —
    /// including how much headroom was left — and a [`GoldenMismatch`] for every
    /// way the check does not hold, *including* the ways it could not run at all.
    ///
    /// # Errors
    ///
    /// - [`GoldenMismatch::DimensionMismatch`] — the rasters are different sizes.
    /// - [`GoldenMismatch::ByteLengthMismatch`] — same size, different byte counts.
    /// - [`GoldenMismatch::PixelsUnavailable`] — a side carries only a hash.
    /// - [`GoldenMismatch::TooManyDifferingPixels`] — too much of the image moved.
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

        let diff = ImageDiff {
            differing_pixels,
            total_pixels: (golden.len() / 4) as u64,
            max_channel_delta: worst_delta,
        };

        if diff.differing_fraction() > tolerance.max_differing_fraction() {
            return Err(GoldenMismatch::TooManyDifferingPixels {
                diff,
                allowed: tolerance.max_differing_fraction(),
            });
        }
        Ok(diff)
    }
}

/// Exact **content identity**: dimensions and content hash.
///
/// Retained pixels are deliberately *not* part of equality. Equality answers the
/// determinism question — same backend, same bytes — which the hash already
/// decides, and keeping it hash-based means a hash-only
/// [`from_parts`](ImageSnapshot::from_parts) snapshot still compares equal to the
/// raster it was digested from. The §7.14 tolerant comparison is
/// [`compare_within`](ImageSnapshot::compare_within), which is a different
/// question and is not an equivalence relation at all — it is not transitive.
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
