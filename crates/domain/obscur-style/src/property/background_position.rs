//! The computed value of CSS `background-position` (spec §5.2, CSS Backgrounds §3).

use crate::property::Length;

/// The computed value of `background-position`: where the (already sized) image
/// tile's origin sits within the painted box, one [`Length`] per axis.
///
/// Each component is stored as a [`Length`] the scene coordinator resolves at
/// paint time against the box, following the standard CSS position formula:
///
/// * a `px` offset places the tile that many pixels from the box's top/left edge;
/// * a `%` offset aligns the image's `pct` point to the box's `pct` point — i.e.
///   the tile origin is `(box_extent − tile_extent) × pct` from the edge (so
///   `50%` centers and `100%` right/bottom-aligns);
/// * the position keywords lower to percentages: `left`/`top` → `0%`,
///   `center` → `50%`, `right`/`bottom` → `100%`.
///
/// The CSS initial is `0% 0%` (top-left), i.e. both axes [`Length::Percent(0.0)`].
///
/// Because [`Length`] carries `f32` data this derives [`PartialEq`] but not
/// [`Eq`]/[`Hash`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BackgroundPosition {
    /// The horizontal position component (from the box's left edge).
    x: Length,
    /// The vertical position component (from the box's top edge).
    y: Length,
}

impl BackgroundPosition {
    /// Builds a position from its per-axis components.
    #[must_use]
    pub const fn new(x: Length, y: Length) -> Self {
        Self { x, y }
    }

    /// The horizontal position component.
    #[must_use]
    pub const fn x(self) -> Length {
        self.x
    }

    /// The vertical position component.
    #[must_use]
    pub const fn y(self) -> Length {
        self.y
    }
}

impl Default for BackgroundPosition {
    /// The CSS initial: `0% 0%` (top-left).
    fn default() -> Self {
        Self {
            x: Length::Percent(0.0),
            y: Length::Percent(0.0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_top_left() {
        let pos = BackgroundPosition::default();
        assert_eq!(pos.x(), Length::Percent(0.0));
        assert_eq!(pos.y(), Length::Percent(0.0));
    }

    #[test]
    fn new_round_trips_axes() {
        let pos = BackgroundPosition::new(Length::Percent(100.0), Length::px(10.0));
        assert_eq!(pos.x(), Length::Percent(100.0));
        assert_eq!(pos.y(), Length::px(10.0));
    }
}
