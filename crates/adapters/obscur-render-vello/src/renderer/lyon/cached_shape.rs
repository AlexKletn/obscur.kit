use obscur_scene::path::Path;

use super::{shape::Shape, stroke_geometry::StrokeGeometry};

pub(super) enum CachedShape {
    Path(Path, Option<StrokeGeometry>),
    Glyph {
        font: u32,
        glyph: u32,
        size: f32,
        oblique: bool,
        bold: bool,
    },
}

impl From<Shape<'_>> for CachedShape {
    fn from(shape: Shape<'_>) -> Self {
        match shape {
            Shape::Path(path, stroke) => Self::Path(path.clone(), stroke.cloned()),
            Shape::Glyph {
                font,
                glyph,
                size,
                oblique,
                bold,
            } => Self::Glyph {
                font,
                glyph,
                size,
                oblique,
                bold,
            },
        }
    }
}

impl CachedShape {
    pub(super) fn borrow(&self) -> Shape<'_> {
        match self {
            Self::Path(path, stroke) => Shape::Path(path, stroke.as_ref()),
            &Self::Glyph {
                font,
                glyph,
                size,
                oblique,
                bold,
            } => Shape::Glyph {
                font,
                glyph,
                size,
                oblique,
                bold,
            },
        }
    }
}
