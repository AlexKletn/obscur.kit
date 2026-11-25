#![doc = include_str!("css_motion.doc/_module.md")]
#![forbid(unsafe_code)]

use obscur_identity::NodeId;
use obscur_motion::timeline::{FillMode, PlaybackDirection, RepeatCount};
use obscur_motion::track::{Easing, Keyframe, KeyframeTrack};
use obscur_motion::transition::TransitionInterruption;
use obscur_motion::value::MotionValue;
use obscur_runtime::UiRuntime;
use obscur_runtime::orchestration::{
    AnimatedKind, CssKeyframeRule, CssMotionRules, CssTransitionRule,
};
use obscur_style::property::PropertyId;
use obscur_style::selector::PseudoState;
use obscur_testkit::fixture::{CounterPanel, HoverBox, TestFixture};
use obscur_testkit::selector::NodePath;

// Four animation unit tests were removed here 2026-07-20 by decision.
// They asserted that an animated value (opacity, and reanimate start-points)
// is written onto the node's COMPUTED style, which is exactly what the spec
// oracle `an_animated_opacity_never_reaches_computed_style` forbids: a motion
// overlay is a paint-time concern and must not mutate computed style. These
// encoded the old coupling and defended it against the fix; the spec suite
// (spec_ch4_motion, spec_ch9_motion) now covers this ground correctly.

/// The arranged width of a node, which is where a motion overlay has to land to
/// be worth anything.
fn box_width(runtime: &UiRuntime, node: NodeId) -> f32 {
    runtime
        .layout_boxes()
        .get(&node)
        .expect("the animated node has a box")
        .size()
        .width
}

/// A linear keyframe track from `from` to `to` over the full `[0, 1]` range.
fn scalar_track(from: f32, to: f32) -> KeyframeTrack {
    KeyframeTrack::from_keyframes(vec![
        Keyframe::new(0.0, MotionValue::scalar(from)),
        Keyframe::new(1.0, MotionValue::scalar(to)),
    ])
}

/// A transitioned property that changes registers a timeline and interpolates
/// across it.
#[test]
fn css_transition_registers_and_interpolates_on_change() {
    let mut runtime = TestFixture::runtime();
    let view = HoverBox::view();
    let sheet = HoverBox::layout_hover_sheet();
    let sheets = std::slice::from_ref(&sheet);
    runtime.set_css_motion(CssMotionRules::from_parts(
        vec![CssTransitionRule::new(
            HoverBox::selectors(),
            PropertyId::Width,
            1.0,
            Easing::Linear,
        )],
        Vec::new(),
    ));

    // Frame 1 (no hover): the transition records its baseline (width 100) but
    // starts nothing — no change yet.
    let _ = runtime.render_now(&view, sheets);
    let child = NodePath::new(&[0]).resolve(&runtime);
    assert_eq!(runtime.active_animations(), 0, "no change => no timeline");
    assert_eq!(box_width(&runtime, child), 100.0);

    // Hover changes the resolved width 100 -> 200: a transition timeline is
    // registered, and at t == 0 the overlaid width is still the old value.
    runtime.set_pseudo_state(child, PseudoState::HOVER);
    let _ = runtime.render_now(&view, sheets);
    assert_eq!(
        runtime.active_animations(),
        1,
        "a changed transitioned property registers a running timeline"
    );
    assert_eq!(
        box_width(&runtime, child),
        100.0,
        "starts from the old value"
    );

    // Advance halfway: the sampled width interpolates linearly to 150.
    runtime.advance_motion(0.5);
    let _ = runtime.render_now(&view, sheets);
    assert_eq!(box_width(&runtime, child), 150.0, "interpolates old->new");
    assert_eq!(runtime.active_animations(), 1, "still running at t = 0.5");

    // Advance to the end: the timeline finishes and the width settles at 200.
    runtime.advance_motion(0.5);
    let _ = runtime.render_now(&view, sheets);
    assert_eq!(
        box_width(&runtime, child),
        200.0,
        "settles at the new value"
    );
    assert_eq!(
        runtime.active_animations(),
        0,
        "finished timeline is pruned"
    );
}

/// A clock-driven timeline keeps scheduling frames after the current request is drained.
#[test]
fn due_frames_keep_css_transition_running_until_it_finishes() {
    let mut runtime = TestFixture::runtime();
    let view = HoverBox::view();
    let sheet = HoverBox::layout_hover_sheet();
    let sheets = std::slice::from_ref(&sheet);
    runtime.set_css_motion(CssMotionRules::from_parts(
        vec![CssTransitionRule::new(
            HoverBox::selectors(),
            PropertyId::Width,
            1.0,
            Easing::Linear,
        )],
        Vec::new(),
    ));
    let _ = runtime.render_now(&view, sheets);
    let node = NodePath::new(&[0]).resolve(&runtime);
    runtime.set_pseudo_state(node, PseudoState::HOVER);
    let _ = runtime.render_now(&view, sheets);

    for expected_width in [125.0, 150.0, 175.0, 200.0] {
        assert!(runtime.has_pending_frame());
        runtime.advance_motion(0.25);
        assert!(runtime.run_due_frame(&view, sheets).is_some());
        assert!((box_width(&runtime, node) - expected_width).abs() < 0.01);
    }
    assert!(!runtime.has_pending_frame());
}

/// Runtime ticks retain the tree, while animation requests from a host can rebuild it.
#[test]
fn reactive_frames_build_only_when_application_requests_a_view() {
    use obscur_runtime::frame::FrameReason;
    use std::{cell::Cell, rc::Rc};

    let mut runtime = TestFixture::runtime();
    let builds = Rc::new(Cell::new(0));
    runtime.set_root_view({
        let builds = Rc::clone(&builds);
        move || {
            builds.set(builds.get() + 1);
            CounterPanel::view()
        }
    });
    runtime.request_frame(FrameReason::UiCommit);
    assert!(runtime.run_due_frame_reactive().is_some());
    let node = NodePath::new(&[0]).resolve(&runtime);
    runtime.animate(
        node,
        PropertyId::Width,
        scalar_track(0.0, 100.0),
        1.0,
        AnimatedKind::LengthPx,
        TransitionInterruption::Blend,
    );

    runtime.advance_motion(0.25);
    assert!(runtime.has_pending_timeline_frame());
    assert!(runtime.run_due_frame_reactive().is_some());
    assert_eq!(builds.get(), 1);
    assert!((box_width(&runtime, node) - 25.0).abs() < 0.01);

    runtime.request_frame(FrameReason::Animation);
    assert!(!runtime.has_pending_timeline_frame());
    assert!(runtime.run_due_frame_reactive().is_some());
    assert_eq!(builds.get(), 2);

    runtime.request_frame(FrameReason::UiCommit);
    assert!(!runtime.has_pending_timeline_frame());
    assert!(runtime.run_due_frame_reactive().is_some());
    assert_eq!(builds.get(), 3);
}

/// A transition rule over a property that never moves starts nothing.
#[test]
fn css_transition_does_not_fire_without_a_change() {
    let mut runtime = TestFixture::runtime();
    let view = HoverBox::view();
    // Only the base rule: the width never changes across frames.
    let sheet = HoverBox::sheet();
    let sheets = std::slice::from_ref(&sheet);
    runtime.set_css_motion(CssMotionRules::from_parts(
        vec![CssTransitionRule::new(
            HoverBox::selectors(),
            PropertyId::Width,
            1.0,
            Easing::Linear,
        )],
        Vec::new(),
    ));

    let _ = runtime.render_now(&view, sheets);
    let _ = runtime.render_now(&view, sheets);
    assert_eq!(
        runtime.active_animations(),
        0,
        "an unchanged property registers no timeline"
    );
}

/// A matched `@keyframes` rule registers its timeline once, not once per frame.
#[test]
fn css_keyframes_rule_registers_a_keyframe_timeline() {
    let mut runtime = TestFixture::runtime();
    let view = HoverBox::view();
    // The base rule sizes the box so it lays out; the animation targets opacity.
    let sheet = HoverBox::sheet();
    let sheets = std::slice::from_ref(&sheet);
    let track = scalar_track(0.0, 1.0);
    runtime.set_css_motion(CssMotionRules::from_parts(
        Vec::new(),
        vec![CssKeyframeRule::new(
            HoverBox::selectors(),
            vec![(PropertyId::Opacity, track)],
            1.0,
        )],
    ));

    // Matching `.box` registers the keyframe timeline once.
    let _ = runtime.render_now(&view, sheets);
    assert_eq!(
        runtime.active_animations(),
        1,
        "an @keyframes-referencing rule registers a keyframe timeline"
    );

    // A second frame must not re-register it (started-set guards it).
    let _ = runtime.render_now(&view, sheets);
    assert_eq!(
        runtime.active_animations(),
        1,
        "registered once, not per frame"
    );
}

/// css-animations-1 §3.9: `backwards` fill applies, during the delay, "the
/// property values defined in the keyframe that will start the first
/// iteration"; `none` applies nothing until the animation runs.
#[test]
fn a_delayed_animation_paints_only_under_a_backwards_fill() {
    // `.box` cascades to width 100; the animation would drive it 0 -> 50.
    let sheet = HoverBox::sheet();
    let sheets = std::slice::from_ref(&sheet);
    let rule = |fill| {
        CssKeyframeRule::new(
            HoverBox::selectors(),
            vec![(PropertyId::Width, scalar_track(0.0, 50.0))],
            1.0,
        )
        .with_playback(
            1.0,
            RepeatCount::Finite(1),
            PlaybackDirection::Forward,
            false,
            fill,
        )
    };

    // Fill `none`: inside the delay the cascade value stands.
    let mut runtime = TestFixture::runtime();
    let view = HoverBox::view();
    runtime.set_css_motion(CssMotionRules::from_parts(
        Vec::new(),
        vec![rule(FillMode::None)],
    ));
    let _ = runtime.render_now(&view, sheets);
    runtime.advance_motion(0.5);
    let _ = runtime.render_now(&view, sheets);
    let child = NodePath::new(&[0]).resolve(&runtime);
    assert_eq!(
        box_width(&runtime, child),
        100.0,
        "no fill: the delay leaves the cascade value alone"
    );

    // Fill `backwards`: the first iteration's start keyframe is applied.
    let mut runtime = TestFixture::runtime();
    runtime.set_css_motion(CssMotionRules::from_parts(
        Vec::new(),
        vec![rule(FillMode::Backwards)],
    ));
    let _ = runtime.render_now(&view, sheets);
    runtime.advance_motion(0.5);
    let _ = runtime.render_now(&view, sheets);
    let child = NodePath::new(&[0]).resolve(&runtime);
    assert_eq!(
        box_width(&runtime, child),
        0.0,
        "backwards fill: the from keyframe applies during the delay"
    );

    // Past the delay the animation runs normally: 0.5s into a 1s cycle.
    runtime.advance_motion(1.0);
    let _ = runtime.render_now(&view, sheets);
    assert_eq!(box_width(&runtime, child), 25.0, "halfway once running");
}

/// An imperatively animated value travels style → motion → layout → paint.
#[test]
fn animated_width_flows_through_style_motion_layout_paint() {
    let mut runtime = TestFixture::runtime();
    let sheet = CounterPanel::height_only_sheet();
    let sheets = std::slice::from_ref(&sheet);
    let view = CounterPanel::view();

    // First frame populates the tree so the node id exists.
    let _ = runtime.render_now(&view, sheets);
    let node = NodePath::new(&[0]).resolve(&runtime);

    // Animate width 0px -> 100px over one logical second.
    runtime.animate(
        node,
        PropertyId::Width,
        scalar_track(0.0, 100.0),
        1.0,
        AnimatedKind::LengthPx,
        TransitionInterruption::Blend,
    );
    assert_eq!(runtime.active_animations(), 1, "one running animation");

    // t = 0: width starts near 0.
    let _ = runtime.render_now(&view, sheets);
    assert!(box_width(&runtime, node) < 0.5, "width starts at ~0");

    // t = 0.5: width is ~50 — the animated value reached the layout box.
    runtime.advance_motion(0.5);
    let _ = runtime.render_now(&view, sheets);
    assert!(
        (box_width(&runtime, node) - 50.0).abs() < 0.5,
        "width is ~50 halfway through the animation"
    );

    // t = 1.0: width is ~100 and the timeline finishes/deactivates.
    runtime.advance_motion(0.5);
    let _ = runtime.render_now(&view, sheets);
    assert!(
        (box_width(&runtime, node) - 100.0).abs() < 0.5,
        "width is ~100 at the end of the animation"
    );
    assert_eq!(
        runtime.active_animations(),
        0,
        "the finished timeline deactivated"
    );

    // A further advance holds 100 (the binding keeps applying the endpoint).
    runtime.advance_motion(0.5);
    let _ = runtime.render_now(&view, sheets);
    assert!(
        (box_width(&runtime, node) - 100.0).abs() < 0.5,
        "the settled animation holds its final value"
    );
    assert_eq!(runtime.active_animations(), 0);
}

/// Starting an animation is itself a reason to schedule a frame.
#[test]
fn animate_requests_an_animation_frame() {
    let mut runtime = TestFixture::runtime();
    let sheet = CounterPanel::height_only_sheet();
    let _ = runtime.render_now(&CounterPanel::view(), std::slice::from_ref(&sheet));
    let node = NodePath::new(&[0]).resolve(&runtime);
    assert!(!runtime.has_pending_frame());

    runtime.animate(
        node,
        PropertyId::Width,
        scalar_track(0.0, 100.0),
        1.0,
        AnimatedKind::LengthPx,
        TransitionInterruption::Blend,
    );
    assert!(
        runtime.has_pending_frame(),
        "starting an animation schedules a frame"
    );
}

#[test]
fn scrubbing_a_timeline_paints_the_sample_and_holds_it_until_resumed() {
    let mut runtime = TestFixture::runtime();
    let view = HoverBox::view();
    let sheet = HoverBox::layout_hover_sheet();
    let sheets = std::slice::from_ref(&sheet);
    let _ = runtime.render_now(&view, sheets);
    let node = NodePath::new(&[0]).resolve(&runtime);
    let timeline = runtime.animate(
        node,
        PropertyId::Width,
        scalar_track(100.0, 200.0),
        1.0,
        AnimatedKind::LengthPx,
        TransitionInterruption::Restart,
    );
    assert!(runtime.set_animation_progress(timeline, 0.75));
    runtime.advance_motion(0.5);
    let _ = runtime.render_now(&view, sheets);
    assert_eq!(box_width(&runtime, node), 175.0);
    assert!(runtime.set_animation_progress(timeline, 0.5));
    assert!(runtime.resume_animation(timeline));
    runtime.advance_motion(0.125);
    let _ = runtime.render_now(&view, sheets);
    assert_eq!(box_width(&runtime, node), 162.5);
}
