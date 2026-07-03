//! A clip applied to subsequent drawing.

use obscur_geometry::{CornerRadii, Rect};

use crate::path::Path;

use super::{ClipShape, Mask};

/// A clip restricting subsequent drawing to a region (spec §5.8).
///
/// A clip is either a geometric [`ClipShape`] or a [`Mask`]. Shape clips are
/// cheap: they confine drawing to a rectangle, rounded rectangle, or path.
/// Mask clips derive coverage from separate source content and may therefore
/// require an offscreen pass. A `Clip` is a backend-independent descriptor and
/// references no GPU or renderer types.
#[derive(Debug, Clone, PartialEq)]
pub enum Clip {
    /// A geometric clip confining drawing to a shape.
    Shape(ClipShape),
    /// A mask clip modulating coverage from source content.
    Mask(Mask),
}

impl Clip {
    /// Creates a rectangular shape clip.
    #[must_use]
    pub const fn rect(rect: Rect) -> Self {
        Self::Shape(ClipShape::rect(rect))
    }

    /// Creates a rounded-rectangular shape clip.
    #[must_use]
    pub const fn rounded_rect(rect: Rect, radii: CornerRadii) -> Self {
        Self::Shape(ClipShape::rounded_rect(rect, radii))
    }

    /// Creates a path shape clip.
    #[must_use]
    pub fn path(path: Path) -> Self {
        Self::Shape(ClipShape::path(path))
    }

    /// Creates a mask clip.
    #[must_use]
    pub const fn mask(mask: Mask) -> Self {
        Self::Mask(mask)
    }

    /// Returns whether this clip is a [`Clip::Mask`].
    #[must_use]
    pub const fn is_mask(&self) -> bool {
        matches!(self, Self::Mask(_))
    }

    /// The clip's axis-aligned bounding rectangle.
    ///
    /// For a shape clip this is the shape's bounds (see [`ClipShape::bounds`]).
    /// For a mask clip this is `None`: a mask's extent depends on its source
    /// content, which is not known to the clip itself. Never panics.
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
