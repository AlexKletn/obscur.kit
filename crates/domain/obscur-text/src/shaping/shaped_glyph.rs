#![doc = include_str!("shaped_glyph.doc/_module.md")]

#[doc = include_str!("shaped_glyph.doc/ShapedGlyph.md")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShapedGlyph {
    #[doc = include_str!("shaped_glyph.doc/glyph_id.md")]
    glyph_id: u32,
    #[doc = include_str!("shaped_glyph.doc/cluster.md")]
    cluster: usize,
    #[doc = include_str!("shaped_glyph.doc/x_advance.md")]
    x_advance: f32,
    #[doc = include_str!("shaped_glyph.doc/x_offset.md")]
    x_offset: f32,
    #[doc = include_str!("shaped_glyph.doc/y_offset.md")]
    y_offset: f32,
}

impl ShapedGlyph {
    #[doc = include_str!("shaped_glyph.doc/new.md")]
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

    #[doc = include_str!("shaped_glyph.doc/with_offset.md")]
    #[must_use]
    pub const fn with_offset(mut self, x: f32, y: f32) -> Self {
        self.x_offset = x;
        self.y_offset = y;
        self
    }

    #[doc = include_str!("shaped_glyph.doc/with_x_advance.md")]
    #[must_use]
    pub const fn with_x_advance(mut self, x_advance: f32) -> Self {
        self.x_advance = x_advance;
        self
    }

    #[doc = include_str!("shaped_glyph.doc/glyph_id_2.md")]
    #[must_use]
    pub const fn glyph_id(&self) -> u32 {
        self.glyph_id
    }

    #[doc = include_str!("shaped_glyph.doc/cluster_2.md")]
    #[must_use]
    pub const fn cluster(&self) -> usize {
        self.cluster
    }

    #[doc = include_str!("shaped_glyph.doc/x_advance_2.md")]
    #[must_use]
    pub const fn x_advance(&self) -> f32 {
        self.x_advance
    }

    #[doc = include_str!("shaped_glyph.doc/x_offset_2.md")]
    #[must_use]
    pub const fn x_offset(&self) -> f32 {
        self.x_offset
    }

    #[doc = include_str!("shaped_glyph.doc/y_offset_2.md")]
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
