//! How a path outline is stroked.

use super::{Brush, LineCap, LineJoin};

/// Describes how a path's outline is stroked (spec §5.1).
///
/// A stroke paints a band of the given `width` centered on the path, filled
/// with `brush`. The open ends of an unclosed path are shaped by `line_cap`,
/// corners are shaped by `line_join`, and sharp miter joins longer than
/// `miter_limit` times half the width fall back to a bevel.
///
/// Dash patterns are a later addition.
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
}

impl Stroke {
    /// The default miter limit (`4.0`), matching common vector conventions.
    pub const DEFAULT_MITER_LIMIT: f32 = 4.0;

    /// Creates a stroke of the given `brush` and `width` with default join
    /// settings: [`LineCap::Butt`], [`LineJoin::Miter`], and a miter limit of
    /// [`Self::DEFAULT_MITER_LIMIT`].
    #[must_use]
    pub fn new(brush: Brush, width: f32) -> Self {
        Self {
            brush,
            width,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            miter_limit: Self::DEFAULT_MITER_LIMIT,
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
}
