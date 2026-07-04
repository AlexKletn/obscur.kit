//! End-to-end coverage for audit S4 (§4.17): a CSS-authored `transition` and
//! `@keyframes` animation, compiled through the real CSS frontend and driven by
//! the runtime's `MotionScheduler`, actually run.
//!
//! This exercises the whole wiring through the public facade only: author CSS ->
//! compile -> build the runtime's [`CssMotionRules`] from the compiled sheet ->
//! install on a [`UiRuntime`] -> render frames and observe the animated value
//! flow through style -> motion -> layout.
#![forbid(unsafe_code)]

use obscur_kit::css::compiler::{CompiledStylesheet, StylesheetCompiler};
use obscur_kit::css::parser::StylesheetParser;
use obscur_kit::css::source::CssSource;
use obscur_kit::geometry::Size;
use obscur_kit::identity::{ComponentId, NodeId, RuntimeId};
use obscur_kit::runtime::UiRuntime;
use obscur_kit::runtime::orchestration::{CssKeyframeRule, CssMotionRules, CssTransitionRule};
use obscur_kit::style::selector::PseudoState;
use obscur_kit::view::view::{CLASS, View, ViewFragment, ViewNode, ViewProperty, ViewValue};

/// Authored CSS exercising both motion frontends at once: a `transition` on
/// `.box` (whose width changes on `:hover`) and a `@keyframes` animation played
/// on `.spinner` via the `animation` shorthand.
const STYLESHEET: &str = "\
.box { width: 100px; height: 50px; transition: width 1s linear; }\n\
.box:hover { width: 200px; }\n\
@keyframes fade { from { opacity: 0; } to { opacity: 1.0; } }\n\
.spinner { width: 10px; height: 10px; animation: fade 1s; }\n\
";

/// Compiles [`STYLESHEET`] through the real CSS frontend.
fn compile() -> CompiledStylesheet {
    let source = CssSource::anonymous(STYLESHEET);
    let parsed = StylesheetParser::new(&source).parse();
    StylesheetCompiler::new().compile(&parsed)
}

/// Repackages a compiled stylesheet's selector-associated motion into the
/// runtime's domain-only [`CssMotionRules`] input — the plumbing a host performs
/// once per (re)compiled sheet. The CSS->track lowering already happened in the
/// frontend; this only maps the fields across the crate boundary.
fn css_motion_rules(compiled: &CompiledStylesheet) -> CssMotionRules {
    let mut transitions = Vec::new();
    let mut keyframes = Vec::new();
    for rule in &compiled.motion_rules {
        for transition in &rule.transitions {
            transitions.push(CssTransitionRule::new(
                rule.selectors.clone(),
                transition.property,
                transition.duration_secs,
                transition.easing,
            ));
        }
        for animation in &rule.animations {
            keyframes.push(CssKeyframeRule::new(
                rule.selectors.clone(),
                animation.tracks.clone(),
                animation.duration_secs,
            ));
        }
    }
    CssMotionRules::from_parts(transitions, keyframes)
}

/// A `.box` element followed by a `.spinner` element, as two top-level nodes.
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
    runtime.tree().get(root).unwrap().children()[0]
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
    let compiled = compile();
    // The frontend produced selector-associated motion: a `.box` transition and
    // a resolved `.spinner` animation.
    assert_eq!(
        compiled.motion_rules.len(),
        2,
        "one motion rule for `.box`, one for `.spinner`"
    );

    let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), Size::new(300.0, 300.0));
    runtime.set_css_motion(css_motion_rules(&compiled));
    let view = view();
    let sheets = std::slice::from_ref(&compiled.matchable);

    // Frame 1: `.spinner` matches its `@keyframes` rule and registers a keyframe
    // timeline immediately; `.box`'s width has not changed, so its transition
    // has not fired yet.
    let _ = runtime.render_now(&view, sheets);
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
    let _ = runtime.render_now(&view, sheets);
    assert_eq!(
        runtime.active_animations(),
        2,
        "the changed transition registered a running timeline alongside the animation"
    );
    assert_eq!(box_width(&runtime, node), 100.0, "transition starts from the old width");

    // Halfway: the width interpolates linearly toward 200.
    runtime.advance_motion(0.5);
    let _ = runtime.render_now(&view, sheets);
    assert_eq!(box_width(&runtime, node), 150.0, "old -> new interpolation");

    // End: both timelines finish and the width settles at 200.
    runtime.advance_motion(0.5);
    let _ = runtime.render_now(&view, sheets);
    assert_eq!(runtime.active_animations(), 0, "finished timelines are pruned");
    assert_eq!(box_width(&runtime, node), 200.0, "settled at the new width");
}
