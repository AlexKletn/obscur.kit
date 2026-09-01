#![doc = include_str!("text_fits_its_own_box.doc/_module.md")]
#![forbid(unsafe_code)]

use obscur_kit::bootstrap::CssPipeline;
use obscur_kit::css::source::CssSource;
use obscur_kit::geometry::Size;
use obscur_kit::identity::{ComponentId, RuntimeId};
use obscur_kit::primitives::Text;
use obscur_kit::runtime::UiRuntime;
use obscur_kit::scene::command::PaintCommand;
use obscur_kit::view::view::{CLASS, View, ViewFragment, ViewNode, ViewProperty, ViewValue};

#[doc = include_str!("text_fits_its_own_box.doc/SENTENCE.md")]
const SENTENCE: &str = "this field asked for no suggestions";

#[doc = include_str!("text_fits_its_own_box.doc/el.md")]
fn el(class: &str, kids: Vec<ViewNode>) -> ViewNode {
    ViewNode::element(ComponentId::from_raw(80))
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text(class.into())))
        .with_children(ViewFragment::from_nodes(kids))
}

#[doc = include_str!("text_fits_its_own_box.doc/painted_runs.md")]
fn painted_runs(css: &str, surface: f32) -> Vec<String> {
    let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), Size::new(surface, 300.0));
    let mut pipeline = CssPipeline::new();
    pipeline.install(&mut runtime, &[CssSource::anonymous(css)]);
    let root = el(
        "row",
        vec![el(
            "label",
            vec![Text::new(SENTENCE.to_owned()).into_view_node()],
        )],
    );
    let view = View::from_fragment(ViewFragment::from_nodes(vec![root]));
    runtime.render_now(&view, pipeline.sheets());
    runtime
        .snapshot()
        .display_list()
        .commands()
        .iter()
        .filter_map(|cmd| match cmd {
            PaintCommand::DrawText { run, .. } => Some(run.text().to_owned()),
            _ => None,
        })
        .collect()
}

#[test]
fn a_line_that_fills_its_box_exactly_is_not_broken_in_two() {
    // Audit 2-D. Layout measures the run once and sizes the text box from that
    // total; paint re-wraps against the very same box, rebuilding the total word
    // by word. The two are the same additions in a different order and `f32` is
    // not associative — the box came out `293.9999`, the running sum reached
    // `294.00003`, and the last word went to a second line.
    //
    // A sentence with room to spare on either side of the box, so nothing here
    // depends on the exact advance of the font in use.
    let runs = painted_runs(".row { } .label { font-size: 14px }", 800.0);
    assert_eq!(
        runs,
        vec![SENTENCE.to_owned()],
        "the whole sentence fits its own box and must paint as one run"
    );
}

#[test]
fn the_same_holds_at_a_size_where_the_error_lands_differently() {
    // 28px is 14px doubled, so the accumulated error doubles with it. The defect
    // showed at both and not at 10px or 16px, which is how a float coincidence
    // behaves and why one size alone would not have pinned it.
    for size in [10, 14, 16, 28] {
        let css = format!(".row {{ }} .label {{ font-size: {size}px }}");
        assert_eq!(
            painted_runs(&css, 1600.0),
            vec![SENTENCE.to_owned()],
            "at {size}px the sentence fits and must not wrap"
        );
    }
}

#[test]
fn a_line_that_genuinely_overflows_still_wraps() {
    // The other half, and the one a tolerance could break: 200px cannot hold a
    // 294px sentence, so it must still come apart.
    let runs = painted_runs(".row { } .label { font-size: 14px }", 200.0);
    assert!(
        runs.len() >= 2,
        "a sentence wider than its box must still wrap: {runs:?}"
    );
    assert_eq!(
        runs.concat().replace(' ', ""),
        SENTENCE.replace(' ', ""),
        "and every word must survive the break"
    );
}

#[test]
fn an_unstyled_span_was_never_affected_and_still_is_not() {
    // The control. Without a rule of its own the sentence always painted as one
    // run, so this asserts the fix changed nothing where nothing was wrong.
    assert_eq!(
        painted_runs(".row { } .label { }", 800.0),
        vec![SENTENCE.to_owned()]
    );
}
