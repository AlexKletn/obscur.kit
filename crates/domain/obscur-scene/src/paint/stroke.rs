//! How a path outline is stroked.

use super::{Brush, LineCap, LineJoin};

#[doc = include_str!("../../docs/api/paint/stroke.doc/Stroke.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct Stroke {
    /// The brush used to paint the stroked band.
    pub brush: Brush,
    /// Stroke width, in user-space units.
    pub width: f32,
    /// End-cap style for open path ends.
    pub line_cap: LineCap,
    /// Corner-join style.
    pub line_join: LineJoin,
    /// Miter length limit, as a multiple of half the stroke width, beyond which
    /// a [`LineJoin::Miter`] join is bevelled.
    pub miter_limit: f32,
    /// The dash pattern: alternating on/off run lengths in user-space units.
    /// Empty (the default) is a solid, continuous stroke.
    pub dash: Box<[f32]>,
    /// The starting phase (in user-space units) into the dash pattern. Ignored
    /// when `dash` is empty.
    pub dash_offset: f32,
}

impl Stroke {
    /// The default miter limit (`4.0`), matching common vector conventions.
    pub const DEFAULT_MITER_LIMIT: f32 = 4.0;

    /// Creates a stroke of the given `brush` and `width` with default join
    /// settings: [`LineCap::Butt`], [`LineJoin::Miter`], a miter limit of
    /// [`Self::DEFAULT_MITER_LIMIT`], and **no** dash (a solid, continuous
    /// stroke).
    #[must_use]
    pub fn new(brush: Brush, width: f32) -> Self {
        Self {
            brush,
            width,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            miter_limit: Self::DEFAULT_MITER_LIMIT,
            dash: Box::default(),
            dash_offset: 0.0,
        }
    }

    /// Returns a copy of this stroke with its line cap set to `cap`.
    #[must_use]
    pub fn with_line_cap(mut self, cap: LineCap) -> Self {
        self.line_cap = cap;
        self
    }

    /// Returns a copy of this stroke with its line join set to `join`.
    #[must_use]
    pub fn with_line_join(mut self, join: LineJoin) -> Self {
        self.line_join = join;
        self
    }

    /// Returns a copy of this stroke with its miter limit set to `limit`.
    #[must_use]
    pub fn with_miter_limit(mut self, limit: f32) -> Self {
        self.miter_limit = limit;
        self
    }

    #[doc = include_str!("../../docs/api/paint/stroke.doc/with_dash.md")]
    #[must_use]
    pub fn with_dash(mut self, pattern: impl Into<Box<[f32]>>, offset: f32) -> Self {
        self.dash = pattern.into();
        self.dash_offset = offset;
        self
    }

    /// Returns the stroke's brush.
    #[must_use]
    pub const fn brush(&self) -> &Brush {
        &self.brush
    }

    /// Returns the stroke width.
    #[must_use]
    pub const fn width(&self) -> f32 {
        self.width
    }

    /// Returns the stroke's end-cap style.
    #[must_use]
    pub const fn line_cap(&self) -> LineCap {
        self.line_cap
    }

    /// Returns the stroke's corner-join style.
    #[must_use]
    pub const fn line_join(&self) -> LineJoin {
        self.line_join
    }

    /// Returns the stroke's miter limit.
    #[must_use]
    pub const fn miter_limit(&self) -> f32 {
        self.miter_limit
    }

    /// Returns the stroke's dash pattern (empty for a solid stroke).
    #[must_use]
    pub fn dash(&self) -> &[f32] {
        &self.dash
    }

    /// Returns the stroke's dash phase offset (unused when [`Self::dash`] is
    /// empty).
    #[must_use]
    pub const fn dash_offset(&self) -> f32 {
        self.dash_offset
    }

    /// Returns whether this stroke carries a non-empty dash pattern.
    #[must_use]
    pub const fn is_dashed(&self) -> bool {
        !self.dash.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::Color;

    #[test]
    fn new_uses_defaults() {
        let s = Stroke::new(Brush::solid(Color::BLACK), 2.0);
        assert_eq!(s.width(), 2.0);
        assert_eq!(s.line_cap(), LineCap::Butt);
        assert_eq!(s.line_join(), LineJoin::Miter);
        assert_eq!(s.miter_limit(), 4.0);
        assert!(s.brush().is_solid());
    }

    #[test]
    fn builders_override() {
        let s = Stroke::new(Brush::solid(Color::BLACK), 2.0)
            .with_line_cap(LineCap::Round)
            .with_line_join(LineJoin::Bevel)
            .with_miter_limit(8.0);
        assert_eq!(s.line_cap(), LineCap::Round);
        assert_eq!(s.line_join(), LineJoin::Bevel);
        assert_eq!(s.miter_limit(), 8.0);
    }

    #[test]
    fn new_stroke_is_solid_not_dashed() {
        let s = Stroke::new(Brush::solid(Color::BLACK), 2.0);
        assert!(!s.is_dashed());
        assert!(s.dash().is_empty());
        assert_eq!(s.dash_offset(), 0.0);
    }

    #[test]
    fn with_dash_roundtrips_pattern_and_offset() {
        let s = Stroke::new(Brush::solid(Color::BLACK), 2.0).with_dash([4.0, 4.0], 1.5);
        assert!(s.is_dashed());
        assert_eq!(s.dash(), &[4.0, 4.0]);
        assert_eq!(s.dash_offset(), 1.5);
    }

    #[test]
    fn empty_dash_pattern_clears_to_solid() {
        let s = Stroke::new(Brush::solid(Color::BLACK), 2.0).with_dash([] as [f32; 0], 0.0);
        assert!(!s.is_dashed());
    }
}
