#![doc = include_str!("harness.doc/_module.md")]
#![allow(dead_code, unreachable_pub)]

use obscur_kit::css::compiler::StylesheetCompiler;
use obscur_kit::css::parser::StylesheetParser;
use obscur_kit::css::source::CssSource;
use obscur_kit::geometry::{Rect, Size};
use obscur_kit::identity::{ComponentId, NodeId, RuntimeId};
use obscur_kit::runtime::UiRuntime;
use obscur_kit::scene::command::PaintCommand;
use obscur_kit::style::cascade::CascadeOrigin;
use obscur_kit::style::matching::Stylesheet;
use obscur_kit::style::property::{PropertyId, StyleColor};
use obscur_kit::view::view::{CLASS, View, ViewFragment, ViewNode, ViewProperty, ViewValue};

#[doc = include_str!("harness.doc/compile.md")]
#[must_use]
pub fn compile(css: &str) -> Stylesheet {
    let src = CssSource::anonymous(css);
    let parsed = StylesheetParser::new(&src).parse();
    StylesheetCompiler::new().compile(&parsed).matchable
}

#[doc = include_str!("harness.doc/compile_in.md")]
#[must_use]
pub fn compile_in(origin: CascadeOrigin, css: &str) -> Stylesheet {
    let sheet = compile(css);
    Stylesheet::new(origin, sheet.rules().to_vec())
}

#[doc = include_str!("harness.doc/el.md")]
#[must_use]
pub fn el(id: u64, class: &str, kids: Vec<ViewNode>) -> ViewNode {
    ViewNode::element(ComponentId::from_raw(id))
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text(class.into())))
        .with_children(ViewFragment::from_nodes(kids))
}

#[doc = include_str!("harness.doc/txt.md")]
#[must_use]
pub fn txt(content: &str) -> ViewNode {
    ViewNode::text(content)
}

#[doc = include_str!("harness.doc/view.md")]
#[must_use]
pub fn view(nodes: Vec<ViewNode>) -> View {
    View::from_fragment(ViewFragment::from_nodes(nodes))
}

#[doc = include_str!("harness.doc/runtime.md")]
#[must_use]
pub fn runtime(w: f32, h: f32) -> UiRuntime {
    UiRuntime::new(RuntimeId::from_raw(1), Size::new(w, h))
}

#[doc = include_str!("harness.doc/drive_sheets.md")]
#[must_use]
pub fn drive_sheets(w: f32, h: f32, sheets: &[Stylesheet], v: &View) -> UiRuntime {
    let mut rt = runtime(w, h);
    let _ = rt.render_now(v, sheets);
    rt
}

#[doc = include_str!("harness.doc/drive.md")]
#[must_use]
pub fn drive(w: f32, h: f32, css: &str, v: &View) -> UiRuntime {
    let sheet = compile(css);
    drive_sheets(w, h, std::slice::from_ref(&sheet), v)
}

#[doc = include_str!("harness.doc/at.md")]
#[must_use]
pub fn at(rt: &UiRuntime, path: &[usize]) -> NodeId {
    let mut node = rt.tree().root().expect("a root after a frame");
    for &i in path {
        let kids = rt
            .tree()
            .get(node)
            .expect("a live node")
            .child_nodes()
            .to_vec();
        assert!(
            i < kids.len(),
            "child index {i} out of range: node has {} children",
            kids.len()
        );
        node = kids[i];
    }
    node
}

#[doc = include_str!("harness.doc/geom.md")]
#[must_use]
pub fn geom(rt: &UiRuntime, node: NodeId) -> (f32, f32, f32, f32) {
    let b = rt
        .layout_boxes()
        .get(&node)
        .copied()
        .expect("the node was arranged");
    (b.origin().x, b.origin().y, b.size().width, b.size().height)
}

#[doc = include_str!("harness.doc/approx.md")]
pub fn approx(actual: f32, expected: f32, what: &str) {
    assert!(
        (actual - expected).abs() < 1e-2,
        "{what}: expected {expected}, got {actual}"
    );
}

#[doc = include_str!("harness.doc/color_of.md")]
#[must_use]
pub fn color_of(rt: &UiRuntime, node: NodeId) -> Option<StyleColor> {
    rt.computed_style(node)?
        .get(PropertyId::Color)
        .and_then(obscur_kit::style::computed::ComputedValue::as_color)
}

#[doc = include_str!("harness.doc/px_of.md")]
#[must_use]
pub fn px_of(rt: &UiRuntime, node: NodeId, property: PropertyId) -> Option<f32> {
    use obscur_kit::style::property::Length;
    match rt.computed_style(node)?.get(property)?.as_length()? {
        Length::Px(p) => Some(p.0),
        _ => None,
    }
}

#[doc = include_str!("harness.doc/transforms.md")]
#[must_use]
pub fn transforms(rt: &UiRuntime) -> Vec<obscur_kit::geometry::AffineTransform> {
    rt.snapshot()
        .display_list()
        .commands()
        .iter()
        .filter_map(|c| match c {
            PaintCommand::PushTransform(t) => Some(*t),
            _ => None,
        })
        .collect()
}

#[doc = include_str!("harness.doc/clip_bounds.md")]
#[must_use]
pub fn clip_bounds(rt: &UiRuntime) -> Vec<Rect> {
    rt.snapshot()
        .display_list()
        .commands()
        .iter()
        .filter_map(|c| match c {
            PaintCommand::PushClip(clip) => clip.bounds(),
            _ => None,
        })
        .collect()
}

#[doc = include_str!("harness.doc/mapped_bounds.md")]
#[must_use]
pub fn mapped_bounds(t: obscur_kit::geometry::AffineTransform, rect: Rect) -> Rect {
    use obscur_kit::geometry::Point;
    let (x0, y0) = (rect.origin.x, rect.origin.y);
    let (x1, y1) = (x0 + rect.size.width, y0 + rect.size.height);
    let corners = [
        t.apply(Point::new(x0, y0)),
        t.apply(Point::new(x1, y0)),
        t.apply(Point::new(x1, y1)),
        t.apply(Point::new(x0, y1)),
    ];
    let minx = corners.iter().fold(f32::INFINITY, |m, p| m.min(p.x));
    let maxx = corners.iter().fold(f32::NEG_INFINITY, |m, p| m.max(p.x));
    let miny = corners.iter().fold(f32::INFINITY, |m, p| m.min(p.y));
    let maxy = corners.iter().fold(f32::NEG_INFINITY, |m, p| m.max(p.y));
    Rect::new(
        obscur_kit::geometry::Point::new(minx, miny),
        Size::new(maxx - minx, maxy - miny),
    )
}
