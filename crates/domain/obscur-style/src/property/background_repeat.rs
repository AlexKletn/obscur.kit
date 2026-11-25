//! The computed value of CSS `background-repeat` (spec §5.2, CSS Backgrounds §3).

use super::BackgroundRepeatAxis;

#[doc = include_str!("../../docs/api/property/background_repeat.doc/BackgroundRepeat.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BackgroundRepeat {
    /// Tile on both axes (the CSS initial). `space` / `round` fold here.
    #[default]
    Repeat,
    /// A single tile, no tiling on either axis.
    NoRepeat,
    /// Tile horizontally only (a single row of tiles).
    RepeatX,
    /// Tile vertically only (a single column of tiles).
    RepeatY,
    /// Independently specified modes for the horizontal and vertical axes.
    Axes(BackgroundRepeatAxis, BackgroundRepeatAxis),
}

impl BackgroundRepeat {
    /// Returns the horizontal and vertical tiling modes.
    #[must_use]
    pub const fn axes(self) -> (BackgroundRepeatAxis, BackgroundRepeatAxis) {
        use BackgroundRepeatAxis::{NoRepeat, Repeat};
        match self {
            Self::Repeat => (Repeat, Repeat),
            Self::NoRepeat => (NoRepeat, NoRepeat),
            Self::RepeatX => (Repeat, NoRepeat),
            Self::RepeatY => (NoRepeat, Repeat),
            Self::Axes(x, y) => (x, y),
        }
    }

    /// Whether the tile repeats along the horizontal (x) axis.
    #[must_use]
    pub const fn repeat_x(self) -> bool {
        !matches!(self.axes().0, BackgroundRepeatAxis::NoRepeat)
    }

    /// Whether the tile repeats along the vertical (y) axis.
    #[must_use]
    pub const fn repeat_y(self) -> bool {
        !matches!(self.axes().1, BackgroundRepeatAxis::NoRepeat)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_repeats_both_axes() {
        let repeat = BackgroundRepeat::default();
        assert_eq!(repeat, BackgroundRepeat::Repeat);
        assert!(repeat.repeat_x());
        assert!(repeat.repeat_y());
    }

    #[test]
    fn axis_flags_match_variant() {
        assert!(!BackgroundRepeat::NoRepeat.repeat_x());
        assert!(!BackgroundRepeat::NoRepeat.repeat_y());
        assert!(BackgroundRepeat::RepeatX.repeat_x());
        assert!(!BackgroundRepeat::RepeatX.repeat_y());
        assert!(!BackgroundRepeat::RepeatY.repeat_x());
        assert!(BackgroundRepeat::RepeatY.repeat_y());
    }
}
