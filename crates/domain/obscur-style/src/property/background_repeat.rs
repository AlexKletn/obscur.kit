//! The computed value of CSS `background-repeat` (spec §5.2, CSS Backgrounds §3).

/// The computed value of `background-repeat`: whether the sized image tile is
/// repeated (tiled) along each axis to fill the painted box.
///
/// The scene coordinator turns this into a pair of per-axis repeat flags
/// (`repeat_x` / `repeat_y`) at paint time:
///
/// * [`BackgroundRepeat::Repeat`] — tile on both axes (the CSS initial).
/// * [`BackgroundRepeat::NoRepeat`] — a single tile, no tiling.
/// * [`BackgroundRepeat::RepeatX`] — tile horizontally only.
/// * [`BackgroundRepeat::RepeatY`] — tile vertically only.
///
/// The CSS `space` and `round` keywords are **folded to** [`BackgroundRepeat::Repeat`]
/// (a documented approximation): their gap-distribution / integer-fit behaviour is
/// not modelled, so they tile like plain `repeat`.
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
}

impl BackgroundRepeat {
    /// Whether the tile repeats along the horizontal (x) axis.
    #[must_use]
    pub const fn repeat_x(self) -> bool {
        matches!(self, Self::Repeat | Self::RepeatX)
    }

    /// Whether the tile repeats along the vertical (y) axis.
    #[must_use]
    pub const fn repeat_y(self) -> bool {
        matches!(self, Self::Repeat | Self::RepeatY)
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
