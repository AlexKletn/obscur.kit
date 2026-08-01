#![doc = include_str!("clip.doc/_module.md")]

use obscur_geometry::{CornerRadii, Rect};

use crate::path::Path;

use super::{ClipShape, Mask};

#[doc = include_str!("clip.doc/Clip.md")]
#[derive(Debug, Clone, PartialEq)]
pub enum Clip {
    #[doc = include_str!("clip.doc/Shape.md")]
    Shape(ClipShape),
    #[doc = include_str!("clip.doc/Mask.md")]
    Mask(Mask),
}

impl Clip {
    #[doc = include_str!("clip.doc/rect.md")]
    #[must_use]
    pub const fn rect(rect: Rect) -> Self {
        Self::Shape(ClipShape::rect(rect))
    }

    #[doc = include_str!("clip.doc/rounded_rect.md")]
    #[must_use]
    pub const fn rounded_rect(rect: Rect, radii: CornerRadii) -> Self {
        Self::Shape(ClipShape::rounded_rect(rect, radii))
    }

    #[doc = include_str!("clip.doc/path.md")]
    #[must_use]
    pub fn path(path: Path) -> Self {
        Self::Shape(ClipShape::path(path))
    }

    #[doc = include_str!("clip.doc/mask.md")]
    #[must_use]
    pub const fn mask(mask: Mask) -> Self {
        Self::Mask(mask)
    }

    #[doc = include_str!("clip.doc/is_mask.md")]
    #[must_use]
    pub const fn is_mask(&self) -> bool {
        matches!(self, Self::Mask(_))
    }

    #[doc = include_str!("clip.doc/bounds.md")]
    #[must_use]
    pub fn bounds(&self) -> Option<Rect> {
        match self {
            Self::Shape(shape) => shape.bounds(),
            Self::Mask(_) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clip::MaskKind;
    use obscur_geometry::{Point, Size};

    fn sample_rect() -> Rect {
        Rect::new(Point::new(3.0, 4.0), Size::new(6.0, 7.0))
    }

    #[test]
    fn rect_clip_is_a_shape_with_rect_bounds() {
        let clip = Clip::rect(sample_rect());
        assert!(!clip.is_mask());
        assert_eq!(clip.bounds(), Some(sample_rect()));
    }

    #[test]
    fn rounded_rect_clip_bounds_is_outer_rect() {
        let clip = Clip::rounded_rect(sample_rect(), CornerRadii::uniform(2.0));
        assert_eq!(clip.bounds(), Some(sample_rect()));
    }

    #[test]
    fn path_clip_bounds_matches_path() {
        let mut path = Path::new();
        path.move_to(Point::new(0.0, 0.0))
            .line_to(Point::new(9.0, 3.0));
        let clip = Clip::path(path.clone());
        assert_eq!(clip.bounds(), path.bounds());
    }

    #[test]
    fn mask_clip_is_mask_with_no_bounds() {
        let clip = Clip::mask(Mask::luminance());
        assert!(clip.is_mask());
        assert_eq!(clip.bounds(), None);
        assert!(matches!(
            clip,
            Clip::Mask(m) if m.kind() == MaskKind::Luminance
        ));
    }
}
