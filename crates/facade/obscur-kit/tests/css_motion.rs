#![doc = include_str!("css_motion.doc/_module.md")]
#![forbid(unsafe_code)]

use obscur_kit::bootstrap::CssPipeline;
use obscur_kit::css::source::{CssSource, SourceId, SourceVersion};
use obscur_kit::geometry::Size;
use obscur_kit::identity::{ComponentId, NodeId, RuntimeId};
use obscur_kit::runtime::UiRuntime;
use obscur_kit::style::selector::PseudoState;
use obscur_kit::view::view::{CLASS, View, ViewFragment, ViewNode, ViewProperty, ViewValue};

#[doc = include_str!("css_motion.doc/STYLESHEET.md")]
const STYLESHEET: &str = "\
.box { width: 100px; height: 50px; transition: width 1s linear; }\n\
.box:hover { width: 200px; }\n\
@keyframes fade { from { opacity: 0; } to { opacity: 1.0; } }\n\
.spinner { width: 10px; height: 10px; animation: fade 1s; }\n\
";

#[doc = include_str!("css_motion.doc/view.md")]
fn view() -> View {
    View::from_fragment(ViewFragment::from_nodes(vec![
        ViewNode::element(ComponentId::from_raw(1))
            .with_prop(ViewProperty::new(CLASS, ViewValue::Text("box".into()))),
        ViewNode::element(ComponentId::from_raw(2))
            .with_prop(ViewProperty::new(CLASS, ViewValue::Text("spinner".into()))),
    ]))
}

fn box_node(runtime: &UiRuntime) -> NodeId {
    let root = runtime.tree().root().expect("a root after the first frame");
    runtime.tree().get(root).unwrap().child_nodes()[0]
}

fn box_width(runtime: &UiRuntime, node: NodeId) -> f32 {
    runtime
        .layout_boxes()
        .get(&node)
        .expect("the sized node has a box")
        .size()
        .width
}

#[test]
fn css_transition_and_keyframes_drive_through_the_scheduler() {
    let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), Size::new(300.0, 300.0));
    let mut css = CssPipeline::new();
    css.install(&mut runtime, &[CssSource::anonymous(STYLESHEET)]);

    // The pipeline carried the frontend's selector-associated motion across: a
    // `.box` transition and a resolved `.spinner` animation.
    assert_eq!(css.motion().transitions().len(), 1);
    assert_eq!(css.motion().keyframes().len(), 1);

    let view = view();
    let sheets = css.sheets().to_vec();

    // Frame 1: `.spinner` matches its `@keyframes` rule and registers a keyframe
    // timeline immediately; `.box`'s width has not changed, so its transition
    // has not fired yet.
    let _ = runtime.render_now(&view, &sheets);
    let node = box_node(&runtime);
    assert_eq!(
        runtime.active_animations(),
        1,
        "the @keyframes animation registered a running timeline"
    );
    assert_eq!(box_width(&runtime, node), 100.0);

    // Hover the box: its computed width changes 100 -> 200, so the transition
    // registers a second timeline. At t == 0 the overlaid width is still 100.
    runtime.set_pseudo_state(node, PseudoState::HOVER);
    let _ = runtime.render_now(&view, &sheets);
    assert_eq!(
        runtime.active_animations(),
        2,
        "the changed transition registered a running timeline alongside the animation"
    );
    assert_eq!(
        box_width(&runtime, node),
        100.0,
        "transition starts from the old width"
    );

    // Halfway: the width interpolates linearly toward 200.
    runtime.advance_motion(0.5);
    let _ = runtime.render_now(&view, &sheets);
    assert_eq!(box_width(&runtime, node), 150.0, "old -> new interpolation");

    // End: both timelines finish and the width settles at 200.
    runtime.advance_motion(0.5);
    let _ = runtime.render_now(&view, &sheets);
    assert_eq!(
        runtime.active_animations(),
        0,
        "finished timelines are pruned"
    );
    assert_eq!(box_width(&runtime, node), 200.0, "settled at the new width");
}

#[doc = include_str!("css_motion.doc/the_pipeline_parses_each_source_once_per_revision.md")]
#[test]
fn the_pipeline_parses_each_source_once_per_revision() {
    let app = |text: &str| CssSource::new(SourceId::from_raw(1), "app.css", text);

    let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), Size::new(300.0, 300.0));
    let mut css = CssPipeline::new();
    let view = view();

    css.install(&mut runtime, &[app(".box { width: 100px; height: 50px; }")]);
    assert_eq!(css.cached_sources(), 1, "the identified source is cached");
    let _ = runtime.render_now(&view, css.sheets());
    let node = box_node(&runtime);
    assert_eq!(box_width(&runtime, node), 100.0);

    // Same id, same (unbumped) revision, different text: a cache hit, so the
    // text is never read and the width does not move.
    css.install(&mut runtime, &[app(".box { width: 999px; height: 50px; }")]);
    let _ = runtime.render_now(&view, css.sheets());
    assert_eq!(
        box_width(&runtime, node),
        100.0,
        "an unbumped revision is the same source: the cached parse won"
    );

    // The owner stamps the next revision: the entry no longer matches, so this
    // one is parsed.
    css.install(
        &mut runtime,
        &[app(".box { width: 999px; height: 50px; }").at_version(SourceVersion::INITIAL.next())],
    );
    let _ = runtime.render_now(&view, css.sheets());
    assert_eq!(box_width(&runtime, node), 999.0, "a new revision re-parsed");
    assert_eq!(css.cached_sources(), 1, "and replaced the entry, not grew");
}
