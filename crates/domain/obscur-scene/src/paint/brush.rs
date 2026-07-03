//! The typed brush that paints a shape.

use crate::color::Color;

use super::{ConicGradient, LinearGradient, RadialGradient, SolidBrush};

/// The typed source of color used to paint a shape (spec §5.1).
///
/// A `Brush` is one of the concrete paint sources: a flat [`SolidBrush`] or one
/// of the gradient brushes. It carries only backend-independent data; a
/// renderer interprets it into device pixels.
///
/// Image and pattern brushes are a later addition.
#[derive(Debug, Clone, PartialEq)]
pub enum Brush {
    /// A flat, single-color fill.
    Solid(SolidBrush),
    /// A linear (axial) gradient.
    Linear(LinearGradient),
    /// A radial (concentric) gradient.
    Radial(RadialGradient),
    /// A conic (angular / sweep) gradient.
    Conic(ConicGradient),
}

impl Brush {
    /// Creates a solid-color brush painting with `color`.
    #[must_use]
    pub const fn solid(color: Color) -> Self {
        Self::Solid(SolidBrush::new(color))
    }

    /// Returns whether this brush is a [`Brush::Solid`] brush.
    #[must_use]
    pub const fn is_solid(&self) -> bool {
        matches!(self, Self::Solid(_))
    }

    /// Returns the inner [`SolidBrush`] if this is a [`Brush::Solid`] brush,
    /// otherwise `None`.
    #[must_use]
    pub const fn as_solid(&self) -> Option<&SolidBrush> {
        match self {
            Self::Solid(brush) => Some(brush),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn solid_constructor_wraps_color() {
        let b = Brush::solid(Color::WHITE);
        assert!(b.is_solid());
        assert_eq!(b.as_solid(), Some(&SolidBrush::new(Color::WHITE)));
    }

    #[test]
    fn non_solid_as_solid_is_none() {
        use obscur_geometry::Point;

        let g = Brush::Radial(RadialGradient::new(Point::ORIGIN, 1.0, Vec::new()));
        assert!(!g.is_solid());
        assert_eq!(g.as_solid(), None);
    }
}
