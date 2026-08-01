#![doc = include_str!("clip_converter.doc/_module.md")]

use obscur_scene::clip::{Clip, ClipShape};
use obscur_scene::path::Path;
use vello::kurbo::{BezPath, Rect as KurboRect, Shape};

use super::path_converter::PathConverter;

#[doc = include_str!("clip_converter.doc/UNBOUNDED_HALF_EXTENT.md")]
const UNBOUNDED_HALF_EXTENT: f64 = 1.0e9;

#[doc = include_str!("clip_converter.doc/ClipConverter.md")]
#[derive(Debug, Default, Clone, Copy)]
pub struct ClipConverter {
    path: PathConverter,
}

impl ClipConverter {
    #[doc = include_str!("clip_converter.doc/new.md")]
    #[must_use]
    pub const fn new(path: PathConverter) -> Self {
        Self { path }
    }

    #[doc = include_str!("clip_converter.doc/shape.md")]
    #[must_use]
    pub fn shape(&self, clip: &Clip) -> BezPath {
        match clip {
            Clip::Shape(ClipShape::Rect(rect)) => self.path.convert(&Path::rect(*rect)),
            Clip::Shape(ClipShape::RoundedRect { rect, radii }) => {
                self.path.convert(&Path::rounded_rect(*rect, *radii))
            }
            Clip::Shape(ClipShape::Path(path)) => self.path.convert(path),
            Clip::Mask(_) => Self::unbounded(),
        }
    }

    #[doc = include_str!("clip_converter.doc/unbounded.md")]
    #[must_use]
    pub fn unbounded() -> BezPath {
        KurboRect::new(
            -UNBOUNDED_HALF_EXTENT,
            -UNBOUNDED_HALF_EXTENT,
            UNBOUNDED_HALF_EXTENT,
            UNBOUNDED_HALF_EXTENT,
        )
        .to_path(0.1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use obscur_geometry::{CornerRadii, Point, Rect, Size};
    use obscur_scene::clip::Mask;
    use vello::kurbo::{PathEl, Point as KurboPoint};

    #[test]
    fn rect_clip_becomes_closed_rectangle() {
        let clip = Clip::rect(Rect::new(Point::new(1.0, 2.0), Size::new(4.0, 6.0)));
        let bez = ClipConverter::default().shape(&clip);
        assert_eq!(bez.elements().len(), 5);
        assert!(matches!(bez.elements()[0], PathEl::MoveTo(_)));
        assert!(matches!(bez.elements()[4], PathEl::ClosePath));
    }

    #[test]
    fn rounded_rect_clip_is_non_empty() {
        let clip = Clip::rounded_rect(
            Rect::new(Point::new(0.0, 0.0), Size::new(20.0, 10.0)),
            CornerRadii::uniform(3.0),
        );
        let bez = ClipConverter::default().shape(&clip);
        assert!(!bez.elements().is_empty());
    }

    /// A mask carries no source and no extent, so it must confine nothing. An
    /// empty shape would clip the content away — the opposite of the CPU backend
    /// (spec §5.2).
    #[test]
    fn mask_clip_confines_nothing() {
        for mask in [Mask::luminance(), Mask::alpha()] {
            let bez = ClipConverter::default().shape(&Clip::mask(mask));
            assert!(!bez.elements().is_empty(), "{mask:?} produced no outline");
            assert!(
                bez.contains(KurboPoint::new(0.0, 0.0)),
                "{mask:?} clipped away the origin"
            );
            assert!(
                bez.contains(KurboPoint::new(4096.0, 4096.0)),
                "{mask:?} clipped away a far corner"
            );
        }
    }

    /// A layer with no clip and a mask clip are the same "nothing is known to
    /// confine this" case, and must resolve identically.
    #[test]
    fn mask_clip_matches_the_unbounded_outline() {
        let bez = ClipConverter::default().shape(&Clip::mask(Mask::alpha()));
        assert_eq!(bez.elements(), ClipConverter::unbounded().elements());
    }
}
