#![doc = include_str!("text_align_paint.doc/_module.md")]
#![forbid(unsafe_code)]

use obscur_kit::css::compiler::StylesheetCompiler;
use obscur_kit::css::parser::StylesheetParser;
use obscur_kit::css::source::CssSource;
use obscur_kit::geometry::Size;
use obscur_kit::identity::{ComponentId, RuntimeId};
use obscur_kit::runtime::UiRuntime;
use obscur_kit::scene::command::PaintCommand;
use obscur_kit::style::matching::Stylesheet;
use obscur_kit::view::view::{CLASS, View, ViewFragment, ViewNode, ViewProperty, ViewValue};

const VIEWPORT: Size = Size {
    width: 400.0,
    height: 200.0,
};

#[doc = include_str!("text_align_paint.doc/sheet.md")]
fn sheet(align: &str) -> Stylesheet {
    let css =
        format!(".box {{ width: 300px; height: 40px; font-size: 16px; text-align: {align} }}");
    let src = CssSource::anonymous(css.as_str());
    let parsed = StylesheetParser::new(&src).parse();
    StylesheetCompiler::new().compile(&parsed).matchable
}

fn view() -> View {
    let node = ViewNode::element(ComponentId::from_raw(70))
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text("box".into())))
        .with_children(ViewFragment::from_nodes(vec![ViewNode::text("abc")]));
    View::from_fragment(ViewFragment::from_nodes(vec![node]))
}

#[doc = include_str!("text_align_paint.doc/painted.md")]
fn painted(rt: &mut UiRuntime, align: &str) -> (f32, f32) {
    let sheet = sheet(align);
    let result = rt.render_now(&view(), std::slice::from_ref(&sheet));
    result
        .snapshot()
        .display_list()
        .commands()
        .iter()
        .find_map(|command| match command {
            PaintCommand::DrawText { origin, run, .. } => Some((origin.x, run.advance())),
            _ => None,
        })
        .expect("the text node paints a run")
}

fn runtime() -> UiRuntime {
    UiRuntime::new(RuntimeId::from_raw(1), VIEWPORT)
}

#[test]
fn left_aligned_text_starts_at_the_content_edge() {
    // The baseline every other case is measured against, and the behaviour that
    // must not move: `start` is the initial value, so this is what every
    // paragraph in the kit does today.
    let (x, _) = painted(&mut runtime(), "left");
    assert!(x.abs() < 0.01, "left-aligned text starts at x=0, got {x}");
}

#[test]
fn right_aligned_text_ends_at_the_containing_block_edge() {
    let mut rt = runtime();
    let (x, advance) = painted(&mut rt, "right");
    assert!(
        ((x + advance) - 300.0).abs() < 0.01,
        "the line's trailing edge is the box's right edge (300); it starts at {x} \
         and advances {advance}"
    );
    assert!(x > 0.0, "and it actually moved");
}

#[test]
fn centred_text_has_equal_slack_on_both_sides() {
    let mut rt = runtime();
    let (x, advance) = painted(&mut rt, "center");
    let right_gap = 300.0 - (x + advance);
    assert!(
        (x - right_gap).abs() < 0.01,
        "leading gap {x} and trailing gap {right_gap} must match"
    );
    assert!(x > 0.0, "and it actually moved");
}

#[doc = include_str!("text_align_paint.doc/the_three_alignments_are_ordered.md")]
#[test]
fn the_three_alignments_are_ordered() {
    let (left, _) = painted(&mut runtime(), "left");
    let (centre, _) = painted(&mut runtime(), "center");
    let (right, _) = painted(&mut runtime(), "right");
    assert!(
        left < centre && centre < right,
        "left {left} < centre {centre} < right {right}"
    );
}

#[doc = include_str!("text_align_paint.doc/alignment_stops_at_the_padding_not_the_border.md")]
#[test]
fn alignment_stops_at_the_padding_not_the_border() {
    let css = ".box { width: 300px; height: 40px; font-size: 16px; \
               padding-left: 20px; padding-right: 20px; text-align: right }";
    let src = CssSource::anonymous(css);
    let parsed = StylesheetParser::new(&src).parse();
    let sheet = StylesheetCompiler::new().compile(&parsed).matchable;

    let mut rt = runtime();
    let result = rt.render_now(&view(), std::slice::from_ref(&sheet));
    let (x, advance) = result
        .snapshot()
        .display_list()
        .commands()
        .iter()
        .find_map(|command| match command {
            PaintCommand::DrawText { origin, run, .. } => Some((origin.x, run.advance())),
            _ => None,
        })
        .expect("a run");

    assert!(
        ((x + advance) - 320.0).abs() < 0.01,
        "the line ends at the content edge (20 + 300 = 320), not 360; \
         it starts at {x} and advances {advance}"
    );
}
