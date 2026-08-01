#![doc = include_str!("radial_shape.doc/_module.md")]

#[doc = include_str!("radial_shape.doc/RadialShape.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RadialShape {
    #[doc = include_str!("radial_shape.doc/Circle.md")]
    Circle,
    #[doc = include_str!("radial_shape.doc/Ellipse.md")]
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
