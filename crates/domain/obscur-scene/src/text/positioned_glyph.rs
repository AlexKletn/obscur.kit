#![doc = include_str!("positioned_glyph.doc/_module.md")]

#[doc = include_str!("positioned_glyph.doc/PositionedGlyph.md")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PositionedGlyph {
    #[doc = include_str!("positioned_glyph.doc/glyph_id.md")]
    glyph_id: u32,
    #[doc = include_str!("positioned_glyph.doc/cluster.md")]
    cluster: usize,
    #[doc = include_str!("positioned_glyph.doc/x.md")]
    x: f32,
    #[doc = include_str!("positioned_glyph.doc/y.md")]
    y: f32,
    #[doc = include_str!("positioned_glyph.doc/advance.md")]
    advance: f32,
}

impl PositionedGlyph {
    #[doc = include_str!("positioned_glyph.doc/new.md")]
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

    #[doc = include_str!("positioned_glyph.doc/glyph_id_2.md")]
    #[must_use]
    pub const fn glyph_id(&self) -> u32 {
        self.glyph_id
    }

    #[doc = include_str!("positioned_glyph.doc/cluster_2.md")]
    #[must_use]
    pub const fn cluster(&self) -> usize {
        self.cluster
    }

    #[doc = include_str!("positioned_glyph.doc/x_2.md")]
    #[must_use]
    pub const fn x(&self) -> f32 {
        self.x
    }

    #[doc = include_str!("positioned_glyph.doc/y_2.md")]
    #[must_use]
    pub const fn y(&self) -> f32 {
        self.y
    }

    #[doc = include_str!("positioned_glyph.doc/advance_2.md")]
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
