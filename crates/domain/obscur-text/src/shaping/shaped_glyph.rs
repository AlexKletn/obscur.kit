//! A single positioned output glyph produced by shaping (spec §5.10).

#[doc = include_str!("../../docs/api/shaping/shaped_glyph.doc/ShapedGlyph.md")]
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

    #[doc = include_str!("../../docs/api/shaping/shaped_glyph.doc/with_x_advance.md")]
    #[must_use]
    pub const fn with_x_advance(mut self, x_advance: f32) -> Self {
        self.x_advance = x_advance;
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
    fn with_x_advance_replaces_only_the_advance() {
        let glyph = ShapedGlyph::new(42, 3, 6.0)
            .with_offset(1.5, -2.0)
            .with_x_advance(10.0);
        assert_eq!(glyph.x_advance(), 10.0);
        // Identity, cluster, and placement offsets are untouched.
        assert_eq!(glyph.glyph_id(), 42);
        assert_eq!(glyph.cluster(), 3);
        assert_eq!(glyph.x_offset(), 1.5);
        assert_eq!(glyph.y_offset(), -2.0);
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
