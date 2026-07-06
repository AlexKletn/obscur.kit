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
use obscur_kit::scene::paint::Brush;
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
// A `border-radius` element renders a rounded (curved) background path, and its
// subtree is bracketed by a clip through the real live pipeline: the display
// list opens a `PushClip`, paints the rounded background and the child, then
// closes with a `PopClip`.
#[test]
fn authored_border_radius_rounds_background_and_clips_subtree() {
    use obscur_kit::scene::path::PathSegment;

    let mut rt = UiRuntime::new(RuntimeId::from_raw(1), Size::new(VIEWPORT, VIEWPORT));
    let sheet = compile(
        ".box{width:100px;height:100px;background-color:#ff0000;border-radius:8px}\
         .inner{width:20px;height:20px;background-color:#00ff00}",
    );
    // A `.box` parent (rounded, background) wrapping a `.inner` child (background).
    let inner = ViewNode::element(ComponentId::from_raw(31))
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text("inner".into())));
    let outer = ViewNode::element(ComponentId::from_raw(30))
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text("box".into())))
        .with_children(ViewFragment::from_nodes(vec![inner]));
    let view = View::from_fragment(ViewFragment::from_nodes(vec![outer]));

    let result = rt.render_now(&view, std::slice::from_ref(&sheet));
    assert!(result.restyled(), "authored CSS resolved computed styles");
    assert!(result.is_balanced(), "the clip bracket is well-formed");

    let commands = result.snapshot().display_list().commands();
    // The clip opens first and closes last, bracketing the whole subtree.
    assert!(
        matches!(commands.first(), Some(PaintCommand::PushClip(_))),
        "the rounded subtree opens with a PushClip"
    );
    assert!(
        matches!(commands.last(), Some(PaintCommand::PopClip)),
        "the rounded subtree closes with a PopClip"
    );

    // The `.box` background is a curved (rounded) path, not a plain rectangle,
    // and the green child fill paints inside the clip (before the PopClip).
    let curved_red = commands.iter().any(|c| match c {
        PaintCommand::DrawPath { path, paint } => {
            let is_red = paint
                .brush()
                .as_solid()
                .is_some_and(|s| (s.color().r - 1.0).abs() < 1e-3 && s.color().g < 1e-3);
            is_red
                && path
                    .segments()
                    .iter()
                    .any(|s| matches!(s, PathSegment::CubicTo { .. }))
        }
        _ => false,
    });
    assert!(curved_red, "the rounded background is a curved path");

    let green = fills_colored(commands, Color::rgba(0.0, 1.0, 0.0, 1.0));
    assert_eq!(green.len(), 1, "the clipped child still paints its background");
}

// An authored `background: linear-gradient(...)` reaches paint through the real
// pipeline: the element's background fill carries a *gradient* brush with the
// authored color stops (spec §5.2), and the gradient axis spans the arranged box.
#[test]
fn authored_linear_gradient_background_paints_gradient_fill() {
    let mut rt = UiRuntime::new(RuntimeId::from_raw(1), Size::new(VIEWPORT, VIEWPORT));
    let sheet =
        compile(".box{width:100px;height:50px;background:linear-gradient(90deg, red, blue)}");
    let result = rt.render_now(&one_box_view(), std::slice::from_ref(&sheet));
    assert!(result.restyled(), "authored CSS resolved computed styles");
    assert!(result.relaid_out(), "the styled tree arranged a layout box");
    assert!(result.is_balanced(), "the display list is well-formed");

    let bounds = rt
        .layout_boxes()
        .get(&box_node(&rt))
        .copied()
        .expect("the box has a layout box")
        .bounds;

    let commands = result.snapshot().display_list().commands();
    // Exactly one background fill, and it is a linear-gradient brush (not solid).
    let gradients: Vec<_> = commands
        .iter()
        .filter_map(|c| match c {
            PaintCommand::DrawPath { paint, .. } => match paint.brush() {
                Brush::Linear(g) => Some(g),
                _ => None,
            },
            _ => None,
        })
        .collect();
    assert_eq!(gradients.len(), 1, "one linear-gradient background fill");
    let g = gradients[0];

    // The authored stops: red at 0.0, blue at 1.0.
    assert_eq!(g.stops().len(), 2);
    assert_eq!(g.stops()[0].offset(), 0.0);
    assert_eq!(g.stops()[0].color(), Color::rgba(1.0, 0.0, 0.0, 1.0));
    assert_eq!(g.stops()[1].offset(), 1.0);
    assert_eq!(g.stops()[1].color(), Color::rgba(0.0, 0.0, 1.0, 1.0));

    // 90deg (to right): the gradient axis runs horizontally across the box at its
    // vertical center.
    let cy = bounds.origin.y + bounds.size.height / 2.0;
    assert!((g.start().x - bounds.origin.x).abs() < 1e-2, "start at left edge");
    assert!((g.start().y - cy).abs() < 1e-2);
    assert!((g.end().x - bounds.right()).abs() < 1e-2, "end at right edge");
    assert!((g.end().y - cy).abs() < 1e-2);
}

// An authored `box-shadow: 4px 4px 8px black` reaches paint through the real
// pipeline: the display list carries a `DrawShadow` with the authored offset,
// blur, and color, painted BEFORE (behind) the element's background fill.
#[test]
fn authored_box_shadow_paints_draw_shadow_behind_background() {
    let mut rt = UiRuntime::new(RuntimeId::from_raw(1), Size::new(VIEWPORT, VIEWPORT));
    let sheet = compile(
        ".box{width:100px;height:50px;background-color:#00ff00;box-shadow:4px 4px 8px black}",
    );
    let result = rt.render_now(&one_box_view(), std::slice::from_ref(&sheet));
    assert!(result.restyled(), "authored CSS resolved computed styles");
    assert!(result.relaid_out(), "the styled tree arranged a layout box");
    assert!(result.is_balanced(), "the display list is well-formed");

    let bounds = rt
        .layout_boxes()
        .get(&box_node(&rt))
        .copied()
        .expect("the box has a layout box")
        .bounds;

    let commands = result.snapshot().display_list().commands();

    // Exactly one DrawShadow, carrying the authored params over the border box.
    let shadow_idx = commands
        .iter()
        .position(|c| matches!(c, PaintCommand::DrawShadow { .. }))
        .expect("an authored box-shadow emits a DrawShadow");
    let PaintCommand::DrawShadow {
        rect,
        color,
        blur,
        spread,
        offset,
        inset,
        ..
    } = &commands[shadow_idx]
    else {
        unreachable!("matched DrawShadow above");
    };
    assert!(rect_approx_eq(*rect, bounds), "shadow over the border box");
    assert_eq!(*color, Color::rgba(0.0, 0.0, 0.0, 1.0), "authored black");
    assert_eq!(*blur, 8.0);
    assert_eq!(*spread, 0.0);
    assert_eq!(*offset, (4.0, 4.0));
    assert!(!*inset, "a drop shadow, not inset");

    // The drop shadow paints BEFORE the background fill (behind the box).
    let fill_idx = commands
        .iter()
        .position(|c| matches!(c, PaintCommand::DrawPath { .. }))
        .expect("the background fill is present");
    assert!(shadow_idx < fill_idx, "drop shadow paints behind the background");
}

// Regression guard: an element with no authored `box-shadow` emits no DrawShadow.
#[test]
fn no_box_shadow_authored_emits_no_draw_shadow() {
    let mut rt = UiRuntime::new(RuntimeId::from_raw(1), Size::new(VIEWPORT, VIEWPORT));
    let sheet = compile(".box{width:40px;height:40px;background-color:#00ff00}");
    let result = rt.render_now(&one_box_view(), std::slice::from_ref(&sheet));

    let commands = result.snapshot().display_list().commands();
    assert!(
        !commands
            .iter()
            .any(|c| matches!(c, PaintCommand::DrawShadow { .. })),
        "no box-shadow authored ⇒ no DrawShadow emitted"
    );
}

// An authored `transform: translate(30px, 40px)` reaches paint through the real
// pipeline: the element's whole subtree paint is bracketed by a `PushTransform` /
// `PopTransform` carrying the composed affine (a pure translation, so the
// transform-origin wrapping cancels), with the background fill drawn inside.
#[test]
fn authored_transform_brackets_subtree_with_push_pop_transform() {
    use obscur_kit::geometry::AffineTransform;

    let mut rt = UiRuntime::new(RuntimeId::from_raw(1), Size::new(VIEWPORT, VIEWPORT));
    let sheet = compile(
        ".box{width:100px;height:50px;background-color:#00ff00;transform:translate(30px, 40px)}",
    );
    let result = rt.render_now(&one_box_view(), std::slice::from_ref(&sheet));
    assert!(result.restyled(), "authored CSS resolved computed styles");
    assert!(result.relaid_out(), "the styled tree arranged a layout box");
    assert!(result.is_balanced(), "the transform bracket is well-formed");

    let commands = result.snapshot().display_list().commands();

    // The transform opens first (outermost) and closes last.
    let Some(PaintCommand::PushTransform(affine)) = commands.first() else {
        panic!("expected a leading PushTransform, got {:?}", commands.first());
    };
    // A pure translation commutes with the origin wrapping: exactly translate(30, 40).
    assert_eq!(*affine, AffineTransform::translate(30.0, 40.0));
    assert!(
        matches!(commands.last(), Some(PaintCommand::PopTransform)),
        "the subtree closes with a PopTransform"
    );

    // The background fill paints inside the bracket (after the push, before the pop).
    let push_idx = 0;
    let pop_idx = commands.len() - 1;
    let fill_idx = commands
        .iter()
        .position(|c| matches!(c, PaintCommand::DrawPath { .. }))
        .expect("the background fill is present");
    assert!(push_idx < fill_idx && fill_idx < pop_idx, "fill is inside the transform");
}

// Regression guard: an element with no authored `transform` emits no transform
// bracket (byte-identical to before transforms existed).
#[test]
fn no_transform_authored_emits_no_transform_bracket() {
    let mut rt = UiRuntime::new(RuntimeId::from_raw(1), Size::new(VIEWPORT, VIEWPORT));
    let sheet = compile(".box{width:40px;height:40px;background-color:#00ff00}");
    let result = rt.render_now(&one_box_view(), std::slice::from_ref(&sheet));

    let commands = result.snapshot().display_list().commands();
    assert!(
        !commands.iter().any(|c| matches!(
            c,
            PaintCommand::PushTransform(_) | PaintCommand::PopTransform
        )),
        "no transform authored ⇒ no transform bracket"
    );
}

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
