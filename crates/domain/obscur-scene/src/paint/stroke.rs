#![doc = include_str!("stroke.doc/_module.md")]

use super::{Brush, LineCap, LineJoin};

#[doc = include_str!("stroke.doc/Stroke.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct Stroke {
    #[doc = include_str!("stroke.doc/brush.md")]
    pub brush: Brush,
    #[doc = include_str!("stroke.doc/width.md")]
    pub width: f32,
    #[doc = include_str!("stroke.doc/line_cap.md")]
    pub line_cap: LineCap,
    #[doc = include_str!("stroke.doc/line_join.md")]
    pub line_join: LineJoin,
    #[doc = include_str!("stroke.doc/miter_limit.md")]
    pub miter_limit: f32,
    #[doc = include_str!("stroke.doc/dash.md")]
    pub dash: Box<[f32]>,
    #[doc = include_str!("stroke.doc/dash_offset.md")]
    pub dash_offset: f32,
}

impl Stroke {
    #[doc = include_str!("stroke.doc/DEFAULT_MITER_LIMIT.md")]
    pub const DEFAULT_MITER_LIMIT: f32 = 4.0;

    #[doc = include_str!("stroke.doc/new.md")]
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

    #[doc = include_str!("stroke.doc/with_line_cap.md")]
    #[must_use]
    pub fn with_line_cap(mut self, cap: LineCap) -> Self {
        self.line_cap = cap;
        self
    }

    #[doc = include_str!("stroke.doc/with_line_join.md")]
    #[must_use]
    pub fn with_line_join(mut self, join: LineJoin) -> Self {
        self.line_join = join;
        self
    }

    #[doc = include_str!("stroke.doc/with_miter_limit.md")]
    #[must_use]
    pub fn with_miter_limit(mut self, limit: f32) -> Self {
        self.miter_limit = limit;
        self
    }

    #[doc = include_str!("stroke.doc/with_dash.md")]
    #[must_use]
    pub fn with_dash(mut self, pattern: impl Into<Box<[f32]>>, offset: f32) -> Self {
        self.dash = pattern.into();
        self.dash_offset = offset;
        self
    }

    #[doc = include_str!("stroke.doc/brush_2.md")]
    #[must_use]
    pub const fn brush(&self) -> &Brush {
        &self.brush
    }

    #[doc = include_str!("stroke.doc/width_2.md")]
    #[must_use]
    pub const fn width(&self) -> f32 {
        self.width
    }

    #[doc = include_str!("stroke.doc/line_cap_2.md")]
    #[must_use]
    pub const fn line_cap(&self) -> LineCap {
        self.line_cap
    }

    #[doc = include_str!("stroke.doc/line_join_2.md")]
    #[must_use]
    pub const fn line_join(&self) -> LineJoin {
        self.line_join
    }

    #[doc = include_str!("stroke.doc/miter_limit_2.md")]
    #[must_use]
    pub const fn miter_limit(&self) -> f32 {
        self.miter_limit
    }

    #[doc = include_str!("stroke.doc/dash_2.md")]
    #[must_use]
    pub fn dash(&self) -> &[f32] {
        &self.dash
    }

    #[doc = include_str!("stroke.doc/dash_offset_2.md")]
    #[must_use]
    pub const fn dash_offset(&self) -> f32 {
        self.dash_offset
    }

    #[doc = include_str!("stroke.doc/is_dashed.md")]
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
