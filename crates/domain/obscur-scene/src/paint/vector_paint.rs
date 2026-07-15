//! `VectorPaint`: a vector shape's colour, possibly deferred to `currentColor`.

use crate::color::Color;

/// How one vector shape is coloured — either a fixed colour, or the CSS
/// `currentColor` keyword left **unresolved** (spec §5.17).
///
/// # Why `currentColor` is not a colour
///
/// Spec §5.17 requires `currentColor` to resolve from the computed `color`
/// "allowing theme/state changes without reparsing". Baking it into a colour at
/// parse time would defeat exactly that: a theme switch changes the computed
/// `color`, and the vector document would have to be parsed again to follow it.
/// So it is carried as its own variant all the way to composition, where the
/// coordinator substitutes the node's computed `color` — the same treatment
/// [`BoxShadow`](https://drafts.csswg.org/css-backgrounds/#box-shadow)'s absent
/// colour already gets in this codebase.
///
/// A parsed vector document is therefore *theme-independent*: one parse serves
/// every colour it will ever be drawn in.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VectorPaint {
    /// A colour fixed by the document itself.
    Solid(Color),
    /// The CSS `currentColor` keyword: resolves to the painting node's computed
    /// `color` at composition time.
    CurrentColor,
}

impl VectorPaint {
    /// The colour this paint draws in when the painting node's computed `color`
    /// is `current`.
    ///
    /// A [`Solid`](Self::Solid) ignores `current` — a colour the document fixed
    /// is not the host's to override.
    #[must_use]
    pub const fn resolve(self, current: Color) -> Color {
        match self {
            Self::Solid(color) => color,
            Self::CurrentColor => current,
        }
    }

    /// Whether this paint defers to the computed `color`.
    #[must_use]
    pub const fn is_current_color(self) -> bool {
        matches!(self, Self::CurrentColor)
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
