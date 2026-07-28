//! A math function is a `<length>`, everywhere one is allowed (CSS Values 4
//! §10.1) — through the whole stack, not only where it parses.
//!
//! The defect these pin was two failures of the same rule, one at each end of
//! the pipeline, and each was invisible on its own:
//!
//! 1. **The frontend** rejected a math function as a component of a 1-4 value box
//!    shorthand. That does not degrade one side — it degrades the *whole*
//!    declaration to `Unsupported`, so `border-radius: min(3vmin, 14px)` left
//!    four square corners.
//! 2. **The computed-value step** kept a `calc()` symbolic even after nothing in
//!    it was unknown. Every consumer that reads a length rather than an
//!    expression — the radius corners, `border-width`, the paint path — then read
//!    nothing and contributed zero.
//!
//! Both are asserted here at the level a user would notice: painted geometry.
//! Neither would have failed a test of the parser alone, which is why these
//! render a frame rather than inspect a stylesheet.
#![forbid(unsafe_code)]

use obscur_kit::css::compiler::StylesheetCompiler;
use obscur_kit::css::parser::StylesheetParser;
use obscur_kit::css::source::CssSource;
use obscur_kit::geometry::Size;
use obscur_kit::identity::{ComponentId, RuntimeId};
use obscur_kit::runtime::UiRuntime;
use obscur_kit::scene::command::PaintCommand;
use obscur_kit::scene::path::PathSegment;
use obscur_kit::style::matching::Stylesheet;
use obscur_kit::view::view::{ID, View, ViewFragment, ViewNode, ViewProperty, ViewValue};

fn compile(css: &str) -> Stylesheet {
    let src = CssSource::anonymous(css);
    let parsed = StylesheetParser::new(&src).parse();
    StylesheetCompiler::new().compile(&parsed).matchable
}

fn view() -> View {
    View::from_fragment(ViewFragment::from_nodes(vec![
        ViewNode::element(ComponentId::from_raw(1))
            .with_prop(ViewProperty::new(ID, ViewValue::Text("box".into())))
            .with_children(ViewFragment::from_nodes(vec![
                ViewNode::element(ComponentId::from_raw(2))
                    .with_prop(ViewProperty::new(ID, ViewValue::Text("child".into()))),
            ])),
    ]))
}

/// The count of curved segments across every painted path in one frame of `css`
/// at a 400×300 viewport. A square-cornered background paints straight lines
/// only, so this is zero exactly when the radius was lost.
fn curves(css: &str) -> usize {
    let sheet = compile(css);
    let mut rt = UiRuntime::new(RuntimeId::from_raw(1), Size::new(400.0, 300.0));
    let result = rt.render_now(&view(), std::slice::from_ref(&sheet));
    result
        .snapshot()
        .display_list()
        .commands()
        .iter()
        .filter_map(|command| match command {
            PaintCommand::DrawPath { path, .. } => Some(path),
            _ => None,
        })
        .flat_map(|path| path.segments().iter())
        .filter(|segment| {
            matches!(
                segment,
                PathSegment::QuadTo { .. } | PathSegment::CubicTo { .. }
            )
        })
        .count()
}

/// The control: the same radius written as a plain length rounds the corners.
/// Without it a stack that rounded *nothing* would still satisfy the case below.
#[test]
fn a_plain_length_radius_rounds_the_corners() {
    let painted = curves(
        "#box { width: 200px; height: 100px; background-color: #123; \
                          border-radius: 12px }",
    );
    assert!(painted > 0, "a 12px radius paints curved corners");
}

/// The case: `min()` picks 12px here (3vmin of a 400×300 viewport is 9px — so
/// the *smaller* term is the vmin one, and the corner is 9px). Either way it is
/// a curve, and before the fix it was four straight lines.
#[test]
fn a_math_function_radius_rounds_the_corners_too() {
    let painted = curves(
        "#box { width: 200px; height: 100px; background-color: #123; \
                          border-radius: min(3vmin, 12px) }",
    );
    assert!(
        painted > 0,
        "a math function is a length: the corners must round"
    );
}

/// The shorthand's per-side rule: a math function in one component must not take
/// the other three down with it. `4px` on the block edges is the tell — if the
/// declaration had degraded, no corner would be round at all.
#[test]
fn one_math_component_does_not_drop_the_whole_shorthand() {
    let painted = curves(
        "#box { width: 200px; height: 100px; background-color: #123; \
                          border-radius: min(3vmin, 12px) 4px }",
    );
    assert!(painted > 0, "the mixed shorthand still rounds");
}

/// The same rule one property over, to pin that the fix was the shared component
/// rule and not a special case for radii: `padding` is space the child starts
/// after.
///
/// The padded box is *inside* the root rather than being it, because the root
/// fills the viewport whatever it asks for — measuring padding on the root would
/// measure the viewport instead.
#[test]
fn a_math_function_padding_reserves_space() {
    let css = "#box { width: 200px; height: 100px; padding: min(4vmin, 30px) } \
               #child { width: 10px; height: 10px }";
    let sheet = compile(css);
    let mut rt = UiRuntime::new(RuntimeId::from_raw(1), Size::new(400.0, 300.0));
    let view = View::from_fragment(ViewFragment::from_nodes(vec![
        ViewNode::element(ComponentId::from_raw(1)).with_children(ViewFragment::from_nodes(vec![
            ViewNode::element(ComponentId::from_raw(2))
                .with_prop(ViewProperty::new(ID, ViewValue::Text("box".into())))
                .with_children(ViewFragment::from_nodes(vec![
                    ViewNode::element(ComponentId::from_raw(3))
                        .with_prop(ViewProperty::new(ID, ViewValue::Text("child".into()))),
                ])),
        ])),
    ]));
    let _ = rt.render_now(&view, std::slice::from_ref(&sheet));
    let root = rt.tree().root().expect("a root after a frame");
    let box_node = rt.tree().get(root).expect("live").child_nodes()[0];
    let padded = rt.layout_boxes().get(&box_node).expect("an arranged box");
    // 4vmin of a 400×300 viewport is 12px, under the 30px ceiling, so the 100px
    // content box grows to a 124px border box. Before the fix the whole
    // declaration was dropped and it stayed 100.
    assert_eq!(padded.bounds.size.height, 124.0);
}
