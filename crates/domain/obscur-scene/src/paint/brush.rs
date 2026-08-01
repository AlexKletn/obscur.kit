#![doc = include_str!("brush.doc/_module.md")]

use crate::color::Color;

use super::{ConicGradient, LinearGradient, RadialGradient, SolidBrush};

#[doc = include_str!("brush.doc/Brush.md")]
#[derive(Debug, Clone, PartialEq)]
pub enum Brush {
    #[doc = include_str!("brush.doc/Solid.md")]
    Solid(SolidBrush),
    #[doc = include_str!("brush.doc/Linear.md")]
    Linear(LinearGradient),
    #[doc = include_str!("brush.doc/Radial.md")]
    Radial(RadialGradient),
    #[doc = include_str!("brush.doc/Conic.md")]
    Conic(ConicGradient),
}

impl Brush {
    #[doc = include_str!("brush.doc/solid.md")]
    #[must_use]
    pub const fn solid(color: Color) -> Self {
        Self::Solid(SolidBrush::new(color))
    }

    #[doc = include_str!("brush.doc/is_solid.md")]
    #[must_use]
    pub const fn is_solid(&self) -> bool {
        matches!(self, Self::Solid(_))
    }

    #[doc = include_str!("brush.doc/as_solid.md")]
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
