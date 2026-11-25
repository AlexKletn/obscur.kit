//! A clip applied to subsequent drawing.

use obscur_geometry::{CornerRadii, Rect};

use crate::paint::SceneImage;
use crate::path::Path;

use super::{ClipShape, Mask};

#[doc = include_str!("../../docs/api/clip/clip.doc/Clip.md")]
#[derive(Debug, Clone, PartialEq)]
pub enum Clip {
    /// A geometric clip confining drawing to a shape.
    Shape(ClipShape),
    /// A mask clip modulating coverage from source content.
    Mask(Mask),
    /// A raster coverage mask positioned in scene-local coordinates.
    ImageMask {
        /// Selects alpha or luminance coverage.
        kind: Mask,
        /// Image supplying the per-pixel coverage.
        image: SceneImage,
        /// Rectangle in clip-local coordinates onto which the image is mapped.
        bounds: Rect,
    },
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

    /// Maps the supplied image to the bounds. Samples outside the image rectangle
    /// have zero coverage. Alpha masks use the image alpha. Luminance masks use
    /// linear-light luminance multiplied by image alpha.
    #[must_use]
    pub const fn image_mask(kind: Mask, image: SceneImage, bounds: Rect) -> Self {
        Self::ImageMask {
            kind,
            image,
            bounds,
        }
    }

    /// Returns whether this clip is a [`Clip::Mask`].
    #[must_use]
    pub const fn is_mask(&self) -> bool {
        matches!(self, Self::Mask(_) | Self::ImageMask { .. })
    }

    #[doc = include_str!("../../docs/api/clip/clip.doc/bounds.md")]
    #[must_use]
    pub fn bounds(&self) -> Option<Rect> {
        match self {
            Self::Shape(shape) => shape.bounds(),
            Self::Mask(_) => None,
            Self::ImageMask { bounds, .. } => Some(*bounds),
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
