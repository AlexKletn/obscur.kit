//! End-to-end verification that **authored border CSS** reaches paint and emits
//! the border fills at the box's edges through the *real* live pipeline (spec CSS
//! Backgrounds and Borders §4).
//!
//! Each test authors CSS text, compiles it through the real `obscur-css` frontend
//! (`source -> parse -> compile -> matchable Stylesheet`), and renders one frame
//! on a [`UiRuntime`] via [`render_now`](UiRuntime::render_now), which runs
//! `reconcile -> match -> cascade -> resolve (ComputedStyles) -> layout
//! (LayoutBoxes) -> compose (DisplayList)`. The assertions read the *actual*
//! [`DisplayList`] the pipeline published (`result.snapshot().display_list()`),
//! never a hand-built scene, so a regression anywhere from the CSS `border`
//! shorthand through the computed-value step to the scene coordinator fails the
//! run.
#![forbid(unsafe_code)]

use obscur_kit::css::compiler::StylesheetCompiler;
use obscur_kit::css::parser::StylesheetParser;
use obscur_kit::css::source::CssSource;
use obscur_kit::geometry::{Rect, Size};
use obscur_kit::identity::{ComponentId, NodeId, RuntimeId};
use obscur_kit::runtime::UiRuntime;
use obscur_kit::scene::color::Color;
use obscur_kit::scene::command::PaintCommand;
use obscur_kit::style::matching::Stylesheet;
use obscur_kit::view::view::{View, ViewFragment, ViewNode, ViewProperty, ViewValue, CLASS};

/// The fixed viewport every test renders under, so geometry is deterministic.
const VIEWPORT: f32 = 300.0;

/// Compiles authored CSS text into a runtime-matchable [`Stylesheet`] through the
/// real CSS frontend (`source -> parse -> compile`).
fn compile(css: &str) -> Stylesheet {
    let src = CssSource::anonymous(css);
    let parsed = StylesheetParser::new(&src).parse();
    StylesheetCompiler::new().compile(&parsed).matchable
}

/// A single element classed `.box`, the border target.
fn one_box_view() -> View {
    let node = ViewNode::element(ComponentId::from_raw(30))
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text("box".into())));
    View::from_fragment(ViewFragment::from_nodes(vec![node]))
}

/// The `.box` element [`NodeId`] after a frame.
fn box_node(rt: &UiRuntime) -> NodeId {
    let root = rt.tree().root().expect("root after a frame");
    rt.tree().get(root).unwrap().children().to_vec()[0]
}

/// The solid-fill `DrawPath` rectangles of `rt`'s last frame whose color equals
/// `want` (within a small tolerance), in paint order.
fn fills_colored(commands: &[PaintCommand], want: Color) -> Vec<Rect> {
    commands
        .iter()
        .filter_map(|c| match c {
            PaintCommand::DrawPath { path, paint } => {
                let color = paint.brush().as_solid()?.color();
                let close = (color.r - want.r).abs() < 1e-3
                    && (color.g - want.g).abs() < 1e-3
                    && (color.b - want.b).abs() < 1e-3
                    && (color.a - want.a).abs() < 1e-3;
                if close {
                    path.bounds()
                } else {
                    None
                }
            }
            _ => None,
        })
        .collect()
}

fn rect_approx_eq(a: Rect, b: Rect) -> bool {
    (a.origin.x - b.origin.x).abs() < 1e-3
        && (a.origin.y - b.origin.y).abs() < 1e-3
        && (a.size.width - b.size.width).abs() < 1e-3
        && (a.size.height - b.size.height).abs() < 1e-3
}

// A bordered element emits four red edge fills covering the 2px ring of its
// border box, at the exact edges the layout phase arranged.
#[test]
fn authored_border_paints_four_red_edge_fills() {
    let mut rt = UiRuntime::new(RuntimeId::from_raw(1), Size::new(VIEWPORT, VIEWPORT));
    let sheet = compile(".box{width:100px;height:50px;border:2px solid #ff0000}");
    let result = rt.render_now(&one_box_view(), std::slice::from_ref(&sheet));
    assert!(result.restyled(), "authored CSS resolved computed styles");
    assert!(result.relaid_out(), "the styled tree arranged a layout box");
    assert!(result.is_balanced(), "the display list is well-formed");

    // The actual arranged border box the pipeline produced.
    let bounds = rt
        .layout_boxes()
        .get(&box_node(&rt))
        .copied()
        .expect("the box has a layout box")
        .bounds;
    let (x, y) = (bounds.origin.x, bounds.origin.y);
    let (w, h) = (bounds.size.width, bounds.size.height);

    let commands = result.snapshot().display_list().commands();
    let red = fills_colored(commands, Color::rgba(1.0, 0.0, 0.0, 1.0));
    assert_eq!(red.len(), 4, "one fill per border side");

    // top, right, bottom, left — the 2px outer ring of the border box.
    let expected = [
        Rect::new(bounds.origin, Size::new(w, 2.0)),
        Rect::new(
            obscur_kit::geometry::Point::new(x + w - 2.0, y + 2.0),
            Size::new(2.0, h - 4.0),
        ),
        Rect::new(
            obscur_kit::geometry::Point::new(x, y + h - 2.0),
            Size::new(w, 2.0),
        ),
        Rect::new(
            obscur_kit::geometry::Point::new(x, y + 2.0),
            Size::new(2.0, h - 4.0),
        ),
    ];
    for (got, want) in red.iter().zip(expected) {
        assert!(
            rect_approx_eq(*got, want),
            "border edge mismatch: got {got:?}, want {want:?}"
        );
    }
}

// An element with no authored border emits no border fill (regression guard): a
// background-only element produces exactly one fill, the background.
#[test]
fn no_border_authored_paints_only_the_background() {
    let mut rt = UiRuntime::new(RuntimeId::from_raw(1), Size::new(VIEWPORT, VIEWPORT));
    let sheet = compile(".box{width:40px;height:40px;background-color:#00ff00}");
    let result = rt.render_now(&one_box_view(), std::slice::from_ref(&sheet));

    let commands = result.snapshot().display_list().commands();
    let draws = commands
        .iter()
        .filter(|c| matches!(c, PaintCommand::DrawPath { .. }))
        .count();
    assert_eq!(draws, 1, "only the background fill, no border rects");
    // The one fill is green (the authored background), not a border.
    let green = fills_colored(commands, Color::rgba(0.0, 1.0, 0.0, 1.0));
    assert_eq!(green.len(), 1);
}

// `border-top: 3px solid blue` paints only the top edge, and its color is blue.
#[test]
fn per_side_border_paints_only_that_edge() {
    let mut rt = UiRuntime::new(RuntimeId::from_raw(1), Size::new(VIEWPORT, VIEWPORT));
    let sheet = compile(".box{width:60px;height:20px;border-top:3px solid blue}");
    let result = rt.render_now(&one_box_view(), std::slice::from_ref(&sheet));

    let commands = result.snapshot().display_list().commands();
    let blue = fills_colored(commands, Color::rgba(0.0, 0.0, 1.0, 1.0));
    assert_eq!(blue.len(), 1, "only the top edge paints");
    let bounds = rt
        .layout_boxes()
        .get(&box_node(&rt))
        .copied()
        .unwrap()
        .bounds;
    assert!(rect_approx_eq(
        blue[0],
        Rect::new(bounds.origin, Size::new(bounds.size.width, 3.0))
    ));
}
