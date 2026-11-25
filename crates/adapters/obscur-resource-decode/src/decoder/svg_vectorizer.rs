//! `SvgVectorizer`: SVG source → a [`SceneVector`], keeping `currentColor` deferred.

mod raw_shape;

use raw_shape::collect;

use obscur_geometry::Size;
use obscur_resources::port::VectorDecoder;
use obscur_resources::state::ResourceError;
use obscur_scene::color::Color;
use obscur_scene::paint::{SceneVector, VectorPaint, VectorShape};

/// Two improbable, *distinct* colours injected as the document's `color` on two
/// separate parses. See [`SvgVectorizer`] for why there are two.
const PROBE_A: &str = "#010203";
const PROBE_B: &str = "#fdfeff";

#[doc = include_str!("../../docs/api/decoder/svg_vectorizer.doc/SvgVectorizer.md")]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct SvgVectorizer;

impl SvgVectorizer {
    /// Creates a vectorizer.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    #[doc = include_str!("../../docs/api/decoder/svg_vectorizer.doc/vectorize.md")]
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
        Some(other) if other != from_a && from_a.a == 1.0 => VectorPaint::CurrentColor,
        Some(other) if other != from_a => VectorPaint::CurrentColorWithOpacity(from_a.a),
        _ => VectorPaint::Solid(from_a),
    }
}

#[doc = include_str!("../../docs/api/decoder/svg_vectorizer.doc/parse_with.md")]
fn parse_with(bytes: &[u8], color: &str) -> Result<usvg::Tree, ResourceError> {
    let options = usvg::Options {
        style_sheet: Some(format!("svg {{ color: {color} }}")),
        ..usvg::Options::default()
    };
    super::SvgDecoder::parse(bytes, &options)
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
