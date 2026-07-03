//! A single positioned output glyph produced by shaping (spec §5.10).

/// One shaped output glyph: a font-specific glyph index paired with its pen
/// advance, placement offsets, and the source byte it maps back to (spec §5.10).
///
/// Shaping is not a one-to-one mapping from characters to glyphs. The
/// [`cluster`](Self::cluster) field records the **byte offset** into the source
/// text this glyph originates from, which is what caret hit-testing walks back
/// through:
///
/// - A ligature collapses several source characters into one glyph, so multiple
///   input bytes fold onto a single glyph and several adjacent glyphs may share
///   the same cluster value.
/// - Conversely one character can decompose into multiple glyphs (e.g. a base
///   plus combining marks), so several glyphs can carry the same cluster.
///
/// Advances and offsets are in logical pixels (already scaled for the requested
/// font size), keeping this type adapter-neutral. Storing `f32` fields means the
/// type derives `PartialEq` but not `Eq`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShapedGlyph {
    /// The font-specific glyph index (not a Unicode code point).
    glyph_id: u32,
    /// The byte offset into the source text this glyph maps back to.
    cluster: usize,
    /// The horizontal pen advance, in logical pixels.
    x_advance: f32,
    /// The horizontal placement offset from the pen, in logical pixels.
    x_offset: f32,
    /// The vertical placement offset from the pen, in logical pixels.
    y_offset: f32,
}

impl ShapedGlyph {
    /// Creates a glyph with the given `glyph_id`, source `cluster`, and
    /// `x_advance`, leaving both placement offsets at `0.0`.
    #[must_use]
    pub const fn new(glyph_id: u32, cluster: usize, x_advance: f32) -> Self {
        Self {
            glyph_id,
            cluster,
            x_advance,
            x_offset: 0.0,
            y_offset: 0.0,
        }
    }

    /// Returns this glyph with its placement offsets set to `(x, y)`.
    #[must_use]
    pub const fn with_offset(mut self, x: f32, y: f32) -> Self {
        self.x_offset = x;
        self.y_offset = y;
        self
    }

    /// Returns the font-specific glyph index.
    #[must_use]
    pub const fn glyph_id(&self) -> u32 {
        self.glyph_id
    }

    /// Returns the byte offset into the source text this glyph maps back to.
    #[must_use]
    pub const fn cluster(&self) -> usize {
        self.cluster
    }

    /// Returns the horizontal pen advance, in logical pixels.
    #[must_use]
    pub const fn x_advance(&self) -> f32 {
        self.x_advance
    }

    /// Returns the horizontal placement offset from the pen, in logical pixels.
    #[must_use]
    pub const fn x_offset(&self) -> f32 {
        self.x_offset
    }

    /// Returns the vertical placement offset from the pen, in logical pixels.
    #[must_use]
    pub const fn y_offset(&self) -> f32 {
        self.y_offset
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_leaves_offsets_zero() {
        let glyph = ShapedGlyph::new(42, 3, 9.6);
        assert_eq!(glyph.glyph_id(), 42);
        assert_eq!(glyph.cluster(), 3);
        assert_eq!(glyph.x_advance(), 9.6);
        assert_eq!(glyph.x_offset(), 0.0);
        assert_eq!(glyph.y_offset(), 0.0);
    }

    #[test]
    fn with_offset_builds_placement() {
        let glyph = ShapedGlyph::new(42, 3, 9.6).with_offset(1.5, -2.0);
        assert_eq!(glyph.x_offset(), 1.5);
        assert_eq!(glyph.y_offset(), -2.0);
        // The advance and identity fields are untouched by the offset builder.
        assert_eq!(glyph.x_advance(), 9.6);
        assert_eq!(glyph.glyph_id(), 42);
    }
}
