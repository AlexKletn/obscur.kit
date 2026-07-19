//! **SPECIFICATION-DERIVED** cases for §4.17 "Motion is a separate subsystem,
//! not part of the CSS style engine" — the invariant behind the §9.6 guardrail
//! "make CSS and motion inseparable".
//!
//! Every expected value was produced by an agent reading the specification with
//! **no access to this repository's source**. A failure here is a defect report
//! about the engine, and must not be "corrected" by editing the expected value.
#![forbid(unsafe_code)]

#[path = "spec_ch9_support/mod.rs"]
mod support;

use obscur_kit::motion::track::KeyframeTrack;
use obscur_kit::motion::transition::TransitionInterruption;
use obscur_kit::motion::value::MotionValue;
use obscur_kit::runtime::orchestration::AnimatedKind;
use obscur_kit::scene::command::PaintCommand;
use obscur_kit::style::property::PropertyId;
use support::{approx, at, compile, el, runtime, view};

const VW: f32 = 1000.0;

/// §4.17 Motion engine — "Motion is a separate subsystem, not part of the CSS
/// style engine." / "Pipeline: 1. style engine resolves base/computed values;
/// 2. motion engine produces presentation values;"
///
/// NG-MOTION-01.
#[test]
fn an_animated_opacity_never_reaches_computed_style() {
    let sheet = compile(".a{width:100px;height:100px;opacity:1;background-color:rgb(255,0,0)}");
    let sheets = [sheet];
    let v = view(vec![el(1, "a", vec![])]);

    let mut rt = runtime(VW, VW);
    let _ = rt.render_now(&v, &sheets);
    let node = at(&rt, &[0]);

    // A 1s opacity track from 1 to 0, driven by the motion subsystem alone.
    let track = KeyframeTrack::from_endpoints(MotionValue::scalar(1.0), MotionValue::scalar(0.0));
    let _ = rt.animate(
        node,
        PropertyId::Opacity,
        track,
        1.0,
        AnimatedKind::Number,
        TransitionInterruption::Restart,
    );
    rt.advance_motion(0.5);
    let _ = rt.render_now(&v, &sheets);
    let node = at(&rt, &[0]);

    let computed = rt
        .computed_style(node)
        .expect("a computed style after the frame")
        .opacity()
        .unwrap_or(1.0);
    approx(
        computed,
        1.0,
        "computed style still reports the authored opacity at t=500ms",
    );

    let painted = rt
        .snapshot()
        .display_list()
        .commands()
        .iter()
        .find_map(|c| match c {
            PaintCommand::PushLayer(layer) => Some(layer.opacity()),
            _ => None,
        })
        .expect("a layer carrying the presentation opacity");
    approx(painted, 0.5, "the scene's painted opacity at t=500ms");
}

/// §4.17 — "Motion is a separate subsystem, not part of the CSS style engine."
///
/// NG-MOTION-02: motion must run with no stylesheet loaded at all.
#[test]
fn motion_runs_without_any_stylesheet() {
    let v = view(vec![el(1, "a", vec![])]);
    let no_sheets: [obscur_kit::style::matching::Stylesheet; 0] = [];

    let mut rt = runtime(VW, VW);
    let _ = rt.render_now(&v, &no_sheets);
    let node = at(&rt, &[0]);

    // translate(0, 0) → translate(100, 0), as a 2D transform track.
    let from = MotionValue::transform([1.0, 0.0, 0.0, 1.0, 0.0, 0.0]);
    let to = MotionValue::transform([1.0, 0.0, 0.0, 1.0, 100.0, 0.0]);
    let track = KeyframeTrack::from_endpoints(from, to);
    let _ = rt.animate(
        node,
        PropertyId::Transform,
        track,
        1.0,
        AnimatedKind::Number,
        TransitionInterruption::Restart,
    );
    rt.advance_motion(0.5);
    let _ = rt.render_now(&v, &no_sheets);

    let transform = rt
        .snapshot()
        .display_list()
        .commands()
        .iter()
        .find_map(|c| match c {
            PaintCommand::PushTransform(t) => Some(*t),
            _ => None,
        })
        .expect("the motion engine produced a presentation transform with no CSS involved");
    let moved = transform.apply(obscur_kit::geometry::Point::new(0.0, 0.0));
    approx(moved.x, 50.0, "presentation translation x at t=500ms");
    approx(moved.y, 0.0, "presentation translation y at t=500ms");
}
