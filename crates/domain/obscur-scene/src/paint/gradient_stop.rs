//! A single color stop along a gradient.

use crate::color::Color;

/// One color stop in a gradient: a [`Color`] positioned at a normalized
/// `offset` along the gradient's axis (spec §5.1).
///
/// The `offset` is conventionally in the `0.0..=1.0` range, where `0.0` is the
/// start of the gradient and `1.0` is its end. Values outside that range are
/// permitted but are not meaningful without an
/// [`ExtendMode`](super::ExtendMode) to interpret them.
///
/// Stops are **ordered by offset**: gradients store a `Vec<GradientStop>` that
/// callers are expected to supply pre-sorted in ascending `offset` order. This
/// type does not sort or validate; it is a plain value.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GradientStop {
    /// Position of the stop along the gradient axis, conventionally `0.0..=1.0`.
    pub offset: f32,
    /// Color at this stop.
    pub color: Color,
}

impl GradientStop {
    /// Creates a gradient stop at `offset` with `color`.
    ///
    /// `offset` is conventionally in `0.0..=1.0`; it is stored as given and not
    /// clamped.
    #[must_use]
    pub const fn new(offset: f32, color: Color) -> Self {
        Self { offset, color }
    }

    /// Returns the stop's position along the gradient axis.
    #[must_use]
    pub const fn offset(self) -> f32 {
        self.offset
    }

    /// Returns the stop's color.
    #[must_use]
    pub const fn color(self) -> Color {
        self.color
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_stores_offset_and_color() {
        let s = GradientStop::new(0.25, Color::WHITE);
        assert_eq!(s.offset(), 0.25);
        assert_eq!(s.color(), Color::WHITE);
    }

    #[test]
    fn equality_by_value() {
        assert_eq!(
            GradientStop::new(0.5, Color::BLACK),
            GradientStop::new(0.5, Color::BLACK)
        );
        assert_ne!(
            GradientStop::new(0.5, Color::BLACK),
            GradientStop::new(0.6, Color::BLACK)
        );
    }
}
