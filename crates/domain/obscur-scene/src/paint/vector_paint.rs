//! `VectorPaint`: a vector shape's colour, possibly deferred to `currentColor`.

use crate::color::Color;

#[doc = include_str!("../../docs/api/paint/vector_paint.doc/VectorPaint.md")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VectorPaint {
    /// A colour fixed by the document itself.
    Solid(Color),
    /// The CSS `currentColor` keyword: resolves to the painting node's computed
    /// `color` at composition time.
    CurrentColor,
    /// The host’s computed color, with its alpha multiplied by the authored paint opacity.
    CurrentColorWithOpacity(f32),
}

impl VectorPaint {
    #[doc = include_str!("../../docs/api/paint/vector_paint.doc/resolve.md")]
    #[must_use]
    pub const fn resolve(self, current: Color) -> Color {
        match self {
            Self::Solid(color) => color,
            Self::CurrentColor => current,
            Self::CurrentColorWithOpacity(opacity) => current.with_alpha(current.a * opacity),
        }
    }

    /// Whether this paint defers to the computed `color`.
    #[must_use]
    pub const fn is_current_color(self) -> bool {
        matches!(self, Self::CurrentColor | Self::CurrentColorWithOpacity(_))
    }
}

#[cfg(test)]
mod tests {
    use super::VectorPaint;
    use crate::color::Color;

    fn red() -> Color {
        Color::rgba(1.0, 0.0, 0.0, 1.0)
    }

    fn blue() -> Color {
        Color::rgba(0.0, 0.0, 1.0, 1.0)
    }

    #[test]
    fn current_color_takes_the_computed_color() {
        assert_eq!(VectorPaint::CurrentColor.resolve(red()), red());
        // The same document drawn in another theme: no reparse, new colour.
        assert_eq!(VectorPaint::CurrentColor.resolve(blue()), blue());
    }

    #[test]
    fn a_solid_paint_is_not_the_hosts_to_override() {
        assert_eq!(VectorPaint::Solid(red()).resolve(blue()), red());
        assert!(!VectorPaint::Solid(red()).is_current_color());
        assert!(VectorPaint::CurrentColor.is_current_color());
    }
}
