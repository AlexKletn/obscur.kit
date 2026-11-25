//! `SceneVector`: a parsed vector document that stays vector through the scene.

use obscur_geometry::Size;

use crate::color::Color;
use crate::paint::{Paint, Stroke, VectorPaint, VectorShape};
use crate::path::Path;

#[doc = include_str!("../../docs/api/paint/scene_vector.doc/SceneVector.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct SceneVector {
    /// The document's intrinsic size, from its `width`/`height`/`viewBox` —
    /// the box `object-fit` scales from (spec §5.16).
    size: Size,
    /// The shapes, in paint order.
    shapes: Vec<VectorShape>,
}

impl SceneVector {
    /// Creates a vector document of intrinsic `size` holding `shapes` in paint
    /// order.
    #[must_use]
    pub fn new(size: Size, shapes: Vec<VectorShape>) -> Self {
        Self { size, shapes }
    }

    /// The document's intrinsic size.
    #[must_use]
    pub const fn size(&self) -> Size {
        self.size
    }

    /// The shapes, in paint order.
    #[must_use]
    pub fn shapes(&self) -> &[VectorShape] {
        &self.shapes
    }

    /// Whether the document draws nothing.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.shapes.is_empty()
    }

    /// Whether any shape defers to the computed `color`, i.e. this document's
    /// appearance follows the theme.
    #[must_use]
    pub fn uses_current_color(&self) -> bool {
        self.shapes.iter().any(|shape| {
            shape.fill.is_some_and(VectorPaint::is_current_color)
                || shape
                    .stroke
                    .is_some_and(|(paint, _)| paint.is_current_color())
        })
    }

    #[doc = include_str!("../../docs/api/paint/scene_vector.doc/resolved.md")]
    #[must_use]
    pub fn resolved(&self, current: Color) -> Vec<(Path, Paint)> {
        let mut out = Vec::with_capacity(self.shapes.len());
        for shape in &self.shapes {
            if let Some(fill) = shape.fill {
                out.push((shape.path.clone(), Paint::solid_fill(fill.resolve(current))));
            }
            if let Some((paint, width)) = shape.stroke {
                out.push((
                    shape.path.clone(),
                    Paint::Stroke(Stroke::new(
                        crate::paint::Brush::solid(paint.resolve(current)),
                        width,
                    )),
                ));
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::{SceneVector, VectorShape};
    use crate::color::Color;
    use crate::paint::{Brush, Paint, VectorPaint};
    use crate::path::Path;
    use obscur_geometry::{Point, Size};

    fn red() -> Color {
        Color::rgba(1.0, 0.0, 0.0, 1.0)
    }

    fn blue() -> Color {
        Color::rgba(0.0, 0.0, 1.0, 1.0)
    }

    fn square() -> Path {
        let mut path = Path::new();
        path.move_to(Point::new(0.0, 0.0));
        path.line_to(Point::new(4.0, 0.0));
        path.line_to(Point::new(4.0, 4.0));
        path.close();
        path
    }

    fn document(fill: Option<VectorPaint>, stroke: Option<(VectorPaint, f32)>) -> SceneVector {
        SceneVector::new(
            Size::new(10.0, 10.0),
            vec![VectorShape {
                path: square(),
                fill,
                stroke,
            }],
        )
    }

    #[test]
    fn a_current_color_fill_follows_the_computed_color_with_no_reparse() {
        // THE §5.17 property: one parsed document, drawn in whatever colour the
        // theme currently says.
        let doc = document(Some(VectorPaint::CurrentColor), None);
        assert!(doc.uses_current_color());

        assert_eq!(doc.resolved(red())[0].1, Paint::solid_fill(red()));
        assert_eq!(doc.resolved(blue())[0].1, Paint::solid_fill(blue()));
    }

    #[test]
    fn a_solid_fill_ignores_the_computed_color() {
        let doc = document(Some(VectorPaint::Solid(red())), None);
        assert!(!doc.uses_current_color());
        assert_eq!(doc.resolved(blue())[0].1, Paint::solid_fill(red()));
    }

    #[test]
    fn a_filled_and_stroked_shape_paints_the_fill_first() {
        let doc = document(
            Some(VectorPaint::Solid(red())),
            Some((VectorPaint::CurrentColor, 2.0)),
        );
        let resolved = doc.resolved(blue());
        assert_eq!(resolved.len(), 2, "one pair per paint, not per shape");
        assert_eq!(resolved[0].1, Paint::solid_fill(red()), "fill first");
        assert_eq!(
            resolved[1].1,
            Paint::Stroke(crate::paint::Stroke::new(Brush::solid(blue()), 2.0)),
            "stroke over it, currentColor resolved"
        );
    }

    #[test]
    fn a_shape_with_neither_paint_draws_nothing() {
        let doc = document(None, None);
        assert!(doc.resolved(red()).is_empty());
        assert!(!doc.is_empty(), "the shape exists; it just paints nothing");
    }

    #[test]
    fn an_empty_document_is_empty() {
        let doc = SceneVector::new(Size::new(1.0, 1.0), Vec::new());
        assert!(doc.is_empty());
        assert!(!doc.uses_current_color());
        assert!(doc.resolved(red()).is_empty());
    }
}
