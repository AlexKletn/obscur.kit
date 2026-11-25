//! The ending-size of a style-local `radial-gradient(...)` (spec §5.2).

use crate::property::Length;

#[doc = include_str!("../../docs/api/property/radial_size.doc/RadialSize.md")]
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum RadialSize {
    /// The `100%` ring reaches the box side/corner **closest** to the center
    /// along each axis (`closest-side`).
    ClosestSide,
    /// The `100%` ring reaches the box corner **closest** to the center
    /// (`closest-corner`).
    ClosestCorner,
    /// The `100%` ring reaches the box side/corner **farthest** from the center
    /// along each axis (`farthest-side`).
    FarthestSide,
    /// The `100%` ring reaches the box corner **farthest** from the center
    /// (`farthest-corner`); the CSS default.
    #[default]
    FarthestCorner,
    /// An explicit radius length (an absolute `px`, or a percentage resolved
    /// against the box at paint time).
    Length(Length),
    /// Explicit horizontal and vertical radii. Percentages use the corresponding axis of the gradient box.
    Ellipse(Length, Length),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_farthest_corner() {
        assert_eq!(RadialSize::default(), RadialSize::FarthestCorner);
    }

    #[test]
    fn length_variant_carries_its_length() {
        assert_eq!(
            RadialSize::Length(Length::px(20.0)),
            RadialSize::Length(Length::px(20.0))
        );
    }
}
