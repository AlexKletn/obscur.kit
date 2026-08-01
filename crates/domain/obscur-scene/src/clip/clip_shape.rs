#![doc = include_str!("clip_shape.doc/_module.md")]

use obscur_geometry::{CornerRadii, Rect};

use crate::path::Path;

#[doc = include_str!("clip_shape.doc/ClipShape.md")]
#[derive(Debug, Clone, PartialEq)]
pub enum ClipShape {
    #[doc = include_str!("clip_shape.doc/Rect.md")]
    Rect(Rect),
    #[doc = include_str!("clip_shape.doc/RoundedRect.md")]
    RoundedRect {
        #[doc = include_str!("clip_shape.doc/rect.md")]
        rect: Rect,
        #[doc = include_str!("clip_shape.doc/radii.md")]
        radii: CornerRadii,
    },
    #[doc = include_str!("clip_shape.doc/Path.md")]
    Path(Path),
}

impl ClipShape {
    #[doc = include_str!("clip_shape.doc/rect_2.md")]
    #[must_use]
    pub const fn rect(rect: Rect) -> Self {
        Self::Rect(rect)
    }

    #[doc = include_str!("clip_shape.doc/rounded_rect.md")]
    #[must_use]
    pub const fn rounded_rect(rect: Rect, radii: CornerRadii) -> Self {
        Self::RoundedRect { rect, radii }
    }

    #[doc = include_str!("clip_shape.doc/path.md")]
    #[must_use]
    pub fn path(path: Path) -> Self {
        Self::Path(path)
    }

    #[doc = include_str!("clip_shape.doc/bounds.md")]
    #[must_use]
    pub fn bounds(&self) -> Option<Rect> {
        match self {
            Self::Rect(rect) | Self::RoundedRect { rect, .. } => Some(*rect),
            Self::Path(path) => path.bounds(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use obscur_geometry::{Point, Size};

    fn sample_rect() -> Rect {
        Rect::new(Point::new(1.0, 2.0), Size::new(10.0, 20.0))
    }

    #[test]
    fn rect_bounds_is_the_rect() {
        let shape = ClipShape::rect(sample_rect());
        assert_eq!(shape.bounds(), Some(sample_rect()));
    }

    #[test]
    fn rounded_rect_bounds_is_the_outer_rect() {
        let shape = ClipShape::rounded_rect(sample_rect(), CornerRadii::uniform(4.0));
        assert_eq!(shape.bounds(), Some(sample_rect()));
    }

    #[test]
    fn path_bounds_delegates_to_path() {
        let mut path = Path::new();
        path.move_to(Point::new(0.0, 0.0))
            .line_to(Point::new(5.0, 8.0));
        let shape = ClipShape::path(path.clone());
        assert_eq!(shape.bounds(), path.bounds());
    }

    #[test]
    fn empty_path_bounds_is_none() {
        let shape = ClipShape::path(Path::new());
        assert_eq!(shape.bounds(), None);
    }

    #[test]
    fn constructors_match_variants() {
        assert!(matches!(ClipShape::rect(sample_rect()), ClipShape::Rect(_)));
        assert!(matches!(
            ClipShape::rounded_rect(sample_rect(), CornerRadii::uniform(1.0)),
            ClipShape::RoundedRect { .. }
        ));
    }
}
