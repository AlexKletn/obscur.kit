use obscur_geometry::Point;
use obscur_scene::color::Color;
use obscur_scene::path::{FillRule, Path};

/// One shape as read out of a `usvg` tree, with its paints still concrete.
pub(super) struct RawShape {
    pub(super) path: Path,
    pub(super) fill: Option<Color>,
    pub(super) stroke: Option<(Color, f32)>,
}

#[doc = include_str!("../../../docs/api/decoder/svg_vectorizer.doc/collect.md")]
pub(super) fn collect(group: &usvg::Group, out: &mut Vec<RawShape>) {
    for node in group.children() {
        match node {
            usvg::Node::Path(path) => {
                let rule = match path.fill().map(usvg::Fill::rule) {
                    Some(usvg::FillRule::EvenOdd) => FillRule::EvenOdd,
                    _ => FillRule::NonZero,
                };
                let fill = path
                    .fill()
                    .and_then(|fill| solid(fill.paint(), fill.opacity().get()));
                let stroke = path.stroke().and_then(|s| {
                    solid(s.paint(), s.opacity().get()).map(|color| (color, s.width().get()))
                });
                out.push(RawShape {
                    path: convert_path(path.data(), rule),
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
/// this layer does not model (see [`SceneVector`](obscur_scene::paint::SceneVector)'s scope note).
fn solid(paint: &usvg::Paint, opacity: f32) -> Option<Color> {
    match paint {
        usvg::Paint::Color(color) => Some(Color::rgba(
            f32::from(color.red) / 255.0,
            f32::from(color.green) / 255.0,
            f32::from(color.blue) / 255.0,
            opacity,
        )),
        _ => None,
    }
}

/// Converts a `usvg`/`tiny-skia` path into the scene's own [`Path`].
fn convert_path(data: &usvg::tiny_skia_path::Path, rule: FillRule) -> Path {
    let mut path = Path::with_fill_rule(rule);
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
