//! `SvgVectorizer`: SVG source → a [`SceneVector`], keeping `currentColor` deferred.

use obscur_geometry::{Point, Size};
use obscur_resources::port::VectorDecoder;
use obscur_resources::state::ResourceError;
use obscur_scene::color::Color;
use obscur_scene::paint::{SceneVector, VectorPaint, VectorShape};
use obscur_scene::path::Path;

/// Two improbable, *distinct* colours injected as the document's `color` on two
/// separate parses. See [`SvgVectorizer`] for why there are two.
const PROBE_A: &str = "#010203";
const PROBE_B: &str = "#fdfeff";

/// Converts SVG source into a [`SceneVector`] whose `currentColor` paints are
/// still unresolved (spec §5.17).
///
/// # The problem
///
/// `usvg` resolves `currentColor` **at parse time**: it reads the element's
/// `color` attribute, falling back to black, and bakes the result into the paint.
/// Nothing in its output distinguishes `fill="currentColor"` from `fill="black"`.
/// Following a theme change would therefore mean re-parsing the document with a
/// different injected colour — which is exactly what §5.17 forbids
/// ("...allowing theme/state changes **without reparsing**").
///
/// # The approach
///
/// Parse **twice**, injecting a different `color` each time via `usvg`'s
/// stylesheet option, and compare the two results shape by shape:
///
/// * a paint that came out **different** was `currentColor` — it followed the
///   injected colour, so it is recorded as [`VectorPaint::CurrentColor`] and
///   resolved per frame from the computed `color`;
/// * a paint that came out **identical** was fixed by the document, and is
///   recorded as [`VectorPaint::Solid`].
///
/// This is exact rather than heuristic: no sentinel colour can be mistaken for an
/// authored one, because detection turns on whether a paint *responded to* the
/// injection, not on what value it holds. The cost is one extra parse per
/// document, paid once at load — never per frame, which is the whole point.
///
/// It also gets a subtle case right for free: an SVG that declares its own
/// `color` (e.g. `<g color="blue"><path fill="currentColor"/></g>`) resolves
/// identically under both probes, so it is recorded as solid blue and the host's
/// computed `color` does **not** override it — matching how a browser treats a
/// nested `color` declaration.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct SvgVectorizer;

impl SvgVectorizer {
    /// Creates a vectorizer.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Parses `bytes` into a [`SceneVector`].
    ///
    /// # Errors
    ///
    /// Returns [`ResourceError::DecodeFailed`] when the source is not a valid SVG
    /// document.
    pub fn vectorize(self, bytes: &[u8]) -> Result<SceneVector, ResourceError> {
        let a = parse_with(bytes, PROBE_A)?;
        let b = parse_with(bytes, PROBE_B)?;

        let mut shapes_a = Vec::new();
        collect(a.root(), &mut shapes_a);
        let mut shapes_b = Vec::new();
        collect(b.root(), &mut shapes_b);

        // The two parses differ only in the injected colour, so they produce the
        // same shapes in the same order; pairing them positionally is sound. If
        // they somehow disagree, fall back to the first parse's paints as solid
        // rather than mispairing.
        let paired = shapes_a.len() == shapes_b.len();
        let shapes = shapes_a
            .iter()
            .enumerate()
            .map(|(index, shape)| VectorShape {
                path: shape.path.clone(),
                fill: shape.fill.map(|fill| {
                    resolve_paint(fill, paired.then(|| shapes_b[index].fill).flatten())
                }),
                stroke: shape.stroke.map(|(paint, width)| {
                    (
                        resolve_paint(
                            paint,
                            paired
                                .then(|| shapes_b[index].stroke)
                                .flatten()
                                .map(|(other, _)| other),
                        ),
                        width,
                    )
                }),
            })
            .collect();

        let size = a.size();
        Ok(SceneVector::new(
            Size::new(size.width(), size.height()),
            shapes,
        ))
    }
}

/// Classifies one paint by whether it responded to the injected colour.
fn resolve_paint(from_a: Color, from_b: Option<Color>) -> VectorPaint {
    match from_b {
        Some(other) if other != from_a => VectorPaint::CurrentColor,
        _ => VectorPaint::Solid(from_a),
    }
}

/// Parses `bytes` with `color` injected as the root's `color` property.
///
/// The stylesheet targets `svg` rather than `*` deliberately: it seeds the root
/// so `currentColor` inherits down, while an element that declares its own colour
/// still wins for its own subtree.
fn parse_with(bytes: &[u8], color: &str) -> Result<usvg::Tree, ResourceError> {
    let options = usvg::Options {
        style_sheet: Some(format!("svg {{ color: {color} }}")),
        ..usvg::Options::default()
    };
    usvg::Tree::from_data(bytes, &options)
        .map_err(|err| ResourceError::DecodeFailed(err.to_string().into()))
}

/// One shape as read out of a `usvg` tree, with its paints still concrete.
struct RawShape {
    path: Path,
    fill: Option<Color>,
    stroke: Option<(Color, f32)>,
}

/// Flattens `group`'s paths into `out` in paint order.
///
/// Non-path nodes (images, nested text, groups' own filters/masks) contribute no
/// geometry here — see [`SceneVector`]'s scope note.
fn collect(group: &usvg::Group, out: &mut Vec<RawShape>) {
    for node in group.children() {
        match node {
            usvg::Node::Path(path) => {
                let fill = path.fill().and_then(|fill| solid(fill.paint()));
                let stroke = path
                    .stroke()
                    .and_then(|s| solid(s.paint()).map(|color| (color, s.width().get())));
                out.push(RawShape {
                    path: convert_path(path.data()),
                    fill,
                    stroke,
                });
            }
            usvg::Node::Group(inner) => collect(inner, out),
            // Images and text carry no vector paint this layer models.
            usvg::Node::Image(_) | usvg::Node::Text(_) => {}
        }
    }
}

/// The colour of a solid `usvg` paint, or [`None`] for a gradient/pattern — which
/// this layer does not model (see [`SceneVector`]'s scope note).
fn solid(paint: &usvg::Paint) -> Option<Color> {
    match paint {
        usvg::Paint::Color(color) => Some(Color::rgba(
            f32::from(color.red) / 255.0,
            f32::from(color.green) / 255.0,
            f32::from(color.blue) / 255.0,
            1.0,
        )),
        _ => None,
    }
}

/// Converts a `usvg`/`tiny-skia` path into the scene's own [`Path`].
fn convert_path(data: &usvg::tiny_skia_path::Path) -> Path {
    let mut path = Path::new();
    for segment in data.segments() {
        match segment {
            usvg::tiny_skia_path::PathSegment::MoveTo(p) => {
                path.move_to(Point::new(p.x, p.y));
            }
            usvg::tiny_skia_path::PathSegment::LineTo(p) => {
                path.line_to(Point::new(p.x, p.y));
            }
            usvg::tiny_skia_path::PathSegment::QuadTo(c, p) => {
                path.quad_to(Point::new(c.x, c.y), Point::new(p.x, p.y));
            }
            usvg::tiny_skia_path::PathSegment::CubicTo(c1, c2, p) => {
                path.cubic_to(
                    Point::new(c1.x, c1.y),
                    Point::new(c2.x, c2.y),
                    Point::new(p.x, p.y),
                );
            }
            usvg::tiny_skia_path::PathSegment::Close => {
                path.close();
            }
        }
    }
    path
}

impl VectorDecoder for SvgVectorizer {
    fn decode_vector(&self, bytes: &[u8]) -> Result<SceneVector, ResourceError> {
        self.vectorize(bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::SvgVectorizer;
    use obscur_scene::color::Color;
    use obscur_scene::paint::VectorPaint;

    fn green() -> Color {
        Color::rgba(0.0, 1.0, 0.0, 1.0)
    }

    fn blue() -> Color {
        Color::rgba(0.0, 0.0, 1.0, 1.0)
    }

    #[test]
    fn a_current_color_fill_survives_parsing_unresolved() {
        // THE §5.17 requirement: the parsed document is colour-independent.
        const SRC: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" width="10" height="10">
            <rect width="4" height="4" fill="currentColor"/>
        </svg>"##;

        let doc = SvgVectorizer::new().vectorize(SRC).expect("valid svg");
        assert_eq!(doc.shapes().len(), 1);
        assert_eq!(doc.shapes()[0].fill, Some(VectorPaint::CurrentColor));
        assert!(doc.uses_current_color());

        // One parse, drawn in whatever the theme says — no reparse involved.
        assert_eq!(
            doc.resolved(green())[0].1,
            obscur_scene::paint::Paint::solid_fill(green())
        );
        assert_eq!(
            doc.resolved(blue())[0].1,
            obscur_scene::paint::Paint::solid_fill(blue())
        );
    }

    #[test]
    fn a_literal_fill_is_solid_and_not_mistaken_for_current_color() {
        const SRC: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" width="10" height="10">
            <rect width="4" height="4" fill="#00FF00"/>
        </svg>"##;

        let doc = SvgVectorizer::new().vectorize(SRC).expect("valid svg");
        assert_eq!(doc.shapes()[0].fill, Some(VectorPaint::Solid(green())));
        assert!(!doc.uses_current_color());
    }

    #[test]
    fn a_documents_own_color_declaration_wins_over_the_host() {
        // `currentColor` under a `color`-declaring group resolves to THAT colour
        // in both probes, so it is solid — the host must not override it. This is
        // how a browser behaves, and it falls out of the two-parse diff for free.
        const SRC: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" width="10" height="10">
            <g color="#0000FF"><rect width="4" height="4" fill="currentColor"/></g>
        </svg>"##;

        let doc = SvgVectorizer::new().vectorize(SRC).expect("valid svg");
        assert_eq!(doc.shapes()[0].fill, Some(VectorPaint::Solid(blue())));
        assert_eq!(
            doc.resolved(green())[0].1,
            obscur_scene::paint::Paint::solid_fill(blue()),
            "the document's own colour is not the host's to override"
        );
    }

    #[test]
    fn a_current_color_stroke_is_detected_too() {
        const SRC: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" width="10" height="10">
            <rect width="4" height="4" fill="none" stroke="currentColor" stroke-width="2"/>
        </svg>"##;

        let doc = SvgVectorizer::new().vectorize(SRC).expect("valid svg");
        assert_eq!(doc.shapes()[0].fill, None, "fill=none carries no paint");
        assert_eq!(
            doc.shapes()[0].stroke,
            Some((VectorPaint::CurrentColor, 2.0))
        );
    }

    #[test]
    fn the_intrinsic_size_comes_from_the_document() {
        const SRC: &[u8] =
            br##"<svg xmlns="http://www.w3.org/2000/svg" width="40" height="20" viewBox="0 0 40 20"><rect width="40" height="20" fill="#0a0"/></svg>"##;
        let doc = SvgVectorizer::new().vectorize(SRC).expect("valid svg");
        assert_eq!(doc.size().width, 40.0);
        assert_eq!(doc.size().height, 20.0);
    }

    #[test]
    fn geometry_survives_as_real_path_segments() {
        // Vector, not pixels: the outline reaches the scene as segments.
        const SRC: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" width="10" height="10">
            <path d="M0 0 L4 0 L4 4 Z" fill="currentColor"/>
        </svg>"##;
        let doc = SvgVectorizer::new().vectorize(SRC).expect("valid svg");
        assert!(
            !doc.shapes()[0].path.segments().is_empty(),
            "the path is vector geometry, not a bitmap"
        );
    }

    #[test]
    fn invalid_source_is_an_error_not_a_panic() {
        assert!(SvgVectorizer::new().vectorize(b"not svg at all").is_err());
    }
}
