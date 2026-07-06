//! A style-local `transform-origin` (spec §5.2).

use crate::property::Length;

/// The CSS `transform-origin`: the point a `transform` pivots about.
///
/// Each component is a [`Length`] — an absolute `px` or a `Percent` of the box's
/// border-box width (`x`) / height (`y`). The `left`/`center`/`right` and
/// `top`/`center`/`bottom` keywords lower to `0%` / `50%` / `100%`. The CSS
/// initial value is `50% 50%` (the box center), exposed as [`TransformOrigin::CENTER`];
/// an unauthored `transform-origin` is left unresolved and the coordinator
/// defaults to the center.
///
/// Because [`Length`] carries `f32` data, this derives [`PartialEq`] but not
/// [`Eq`]/[`Hash`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TransformOrigin {
    /// The horizontal origin (a `px` or a percentage of the box width).
    x: Length,
    /// The vertical origin (a `px` or a percentage of the box height).
    y: Length,
}

impl TransformOrigin {
    /// The initial `transform-origin`: `50% 50%` (the box center).
    pub const CENTER: Self = Self {
        x: Length::Percent(50.0),
        y: Length::Percent(50.0),
    };

    /// Creates a transform origin from its horizontal and vertical components.
    #[must_use]
    pub const fn new(x: Length, y: Length) -> Self {
        Self { x, y }
    }

    /// Returns the horizontal origin component.
    #[must_use]
    pub const fn x(&self) -> Length {
        self.x
    }

    /// Returns the vertical origin component.
    #[must_use]
    pub const fn y(&self) -> Length {
        self.y
    }
}

impl Default for TransformOrigin {
    /// The default is the CSS initial [`TransformOrigin::CENTER`].
    fn default() -> Self {
        Self::CENTER
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn center_is_fifty_percent() {
        assert_eq!(TransformOrigin::CENTER.x(), Length::Percent(50.0));
        assert_eq!(TransformOrigin::CENTER.y(), Length::Percent(50.0));
    }

    #[test]
    fn stores_components() {
        let o = TransformOrigin::new(Length::px(10.0), Length::Percent(100.0));
        assert_eq!(o.x(), Length::px(10.0));
        assert_eq!(o.y(), Length::Percent(100.0));
    }
}
