//! The shape of a style-local `radial-gradient(...)` (spec §5.2).

#[doc = include_str!("../../docs/api/property/radial_shape.doc/RadialShape.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RadialShape {
    /// A circular ending shape (a single radius on both axes).
    Circle,
    /// An elliptical ending shape (the CSS default).
    #[default]
    Ellipse,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_ellipse() {
        assert_eq!(RadialShape::default(), RadialShape::Ellipse);
    }
}
