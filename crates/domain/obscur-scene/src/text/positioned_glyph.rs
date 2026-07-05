//! A single positioned glyph carried in a [`GlyphRun`](super::GlyphRun).

/// One shaped, positioned glyph in a [`GlyphRun`](super::GlyphRun): a glyph
/// identifier paired with the pen position it occupies (spec §5.10, §5.2).
///
/// This is the backend-independent unit a [`DrawText`] command carries. Shaping
/// has already run (in the `obscur-text` domain), so a renderer never re-shapes:
/// it maps [`glyph_id`](Self::glyph_id) to an outline and draws it at
/// [`x`](Self::x)/[`y`](Self::y). Because both the GPU and the software backend
/// consume the very same positioned glyphs, they cannot diverge on text layout —
/// only on rasterization (spec §5.2 determinism boundary).
///
/// - [`x`](Self::x)/[`y`](Self::y) are the glyph's pen position **relative to the
///   run's origin**, in logical pixels; `y` is measured at the text baseline.
/// - [`advance`](Self::advance) is the glyph's horizontal advance, letting a
///   software backend that lacks real glyph outlines approximate the glyph with a
///   coverage cell of this width.
/// - [`glyph_id`](Self::glyph_id) is a **font glyph index** — the same contract as
///   the domain
///   [`ShapedGlyph`](../../../obscur_text/shaping/struct.ShapedGlyph.html). A real
///   shaper (the HarfRust adapter) emits real, font-specific glyph indices, so a
///   renderer draws them directly with no remap; the reference
///   [`MockTextShaper`](../../../obscur_text/shaping/struct.MockTextShaper.html)
///   emits the Unicode scalar value as a placeholder index for headless
///   determinism.
/// - [`cluster`](Self::cluster) is the byte offset into the run's source text the
///   glyph maps back to (for caret hit-testing / accessibility).
///
/// Storing `f32` fields means the type derives `PartialEq` but not `Eq`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PositionedGlyph {
    /// The font glyph index (the mock shaper emits the Unicode scalar as a
    /// placeholder index).
    glyph_id: u32,
    /// The byte offset into the run's source text this glyph maps back to.
    cluster: usize,
    /// The pen x position relative to the run origin, in logical pixels.
    x: f32,
    /// The baseline y position relative to the run origin, in logical pixels.
    y: f32,
    /// The glyph's horizontal advance, in logical pixels.
    advance: f32,
}

impl PositionedGlyph {
    /// Creates a positioned glyph with the given identity, source `cluster`, pen
    /// position `(x, y)` relative to the run origin, and horizontal `advance`.
    #[must_use]
    pub const fn new(glyph_id: u32, cluster: usize, x: f32, y: f32, advance: f32) -> Self {
        Self {
            glyph_id,
            cluster,
            x,
            y,
            advance,
        }
    }

    /// Returns the font glyph index (the mock shaper emits the Unicode scalar as
    /// a placeholder).
    #[must_use]
    pub const fn glyph_id(&self) -> u32 {
        self.glyph_id
    }

    /// Returns the byte offset into the run's source text this glyph maps back to.
    #[must_use]
    pub const fn cluster(&self) -> usize {
        self.cluster
    }

    /// Returns the pen x position relative to the run origin, in logical pixels.
    #[must_use]
    pub const fn x(&self) -> f32 {
        self.x
    }

    /// Returns the baseline y position relative to the run origin, in logical
    /// pixels.
    #[must_use]
    pub const fn y(&self) -> f32 {
        self.y
    }

    /// Returns the glyph's horizontal advance, in logical pixels.
    #[must_use]
    pub const fn advance(&self) -> f32 {
        self.advance
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accessors_return_constructor_fields() {
        let glyph = PositionedGlyph::new(97, 0, 3.0, 16.0, 9.6);
        assert_eq!(glyph.glyph_id(), 97);
        assert_eq!(glyph.cluster(), 0);
        assert_eq!(glyph.x(), 3.0);
        assert_eq!(glyph.y(), 16.0);
        assert_eq!(glyph.advance(), 9.6);
    }

    #[test]
    fn equal_glyphs_compare_equal() {
        assert_eq!(
            PositionedGlyph::new(1, 2, 3.0, 4.0, 5.0),
            PositionedGlyph::new(1, 2, 3.0, 4.0, 5.0),
        );
    }
}
