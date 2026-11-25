//! A single positioned glyph carried in a [`GlyphRun`](super::GlyphRun).

#[doc = include_str!("../../docs/api/text/positioned_glyph.doc/PositionedGlyph.md")]
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
