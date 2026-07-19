//! **SPECIFICATION-DERIVED** time and animation-scheduling cases for obscur.kit
//! §3.5 ("animation progress is sampled from time/value, never slowed to match
//! missed frames"; "active animations continue requesting frames") and §3.3
//! ("Runtime adapters provide `Clock`").
//!
//! Every expected value below was produced by an agent reading obscur.kit §3
//! with **no access to this repository's source**. A failure here is a defect
//! report about the runtime, and must not be "corrected" by editing the expected
//! value.
#![forbid(unsafe_code)]

#[path = "spec_ch3_support/mod.rs"]
mod support;

use obscur_kit::identity::NodeId;
use obscur_kit::motion::track::{Keyframe, KeyframeTrack};
use obscur_kit::motion::transition::TransitionInterruption;
use obscur_kit::motion::value::MotionValue;
use obscur_kit::runtime::UiRuntime;
use obscur_kit::runtime::orchestration::AnimatedKind;
use obscur_kit::style::matching::Stylesheet;
use obscur_kit::style::property::PropertyId;
use obscur_kit::view::view::View;
use support::{VSYNC, approx, at, drain_frames, el, geom, runtime, view};

const CSS: &str = ".page { width: 600px; height: 200px } \
                   .box { width: 0px; height: 20px; background-color: #ff0000 }";

fn scene() -> View {
    view(vec![el(1, "page", vec![el(2, "box", vec![])])])
}

/// A runtime with the scene rendered once, plus the animated node.
fn started(duration: f32, from: f32, to: f32) -> (UiRuntime, Stylesheet, NodeId) {
    let sheet = support::compile(CSS);
    let v = scene();
    let mut rt = runtime(600.0, 200.0);
    let _ = rt.render_now(&v, std::slice::from_ref(&sheet));
    let node = at(&rt, &[0, 0]);
    rt.animate(
        node,
        PropertyId::Width,
        KeyframeTrack::from_keyframes(vec![
            Keyframe::new(0.0, MotionValue::scalar(from)),
            Keyframe::new(1.0, MotionValue::scalar(to)),
        ]),
        duration,
        AnimatedKind::LengthPx,
        TransitionInterruption::Blend,
    );
    (rt, sheet, node)
}

/// §3.5 Rules — "animation progress is sampled from time/value, never slowed to
/// match missed frames".
///
/// A 1000 ms 0→100 animation stalled from t=0 to t=900 with no frames presented
/// samples **90** in the next frame — not 1.67 (one 16 ms step) and not 100.
#[test]
fn a_stalled_animation_samples_from_the_clock_not_from_the_frame_count() {
    let (mut rt, sheet, _node) = started(1.0, 0.0, 100.0);
    let sheets = std::slice::from_ref(&sheet);
    let v = scene();

    // Present the t = 0 frame.
    let _ = rt.render_now(&v, sheets);
    approx(geom(&rt, at(&rt, &[0, 0])).2, 0.0, "the width at t=0");

    // The stall: the clock reaches t = 900 ms with NOT ONE frame in between.
    rt.advance_motion(0.9);
    let _ = rt.render_now(&v, sheets);

    approx(
        geom(&rt, at(&rt, &[0, 0])).2,
        90.0,
        "progress is 0.9 of the way because 900 ms of TIME passed — not 1/60th \
         because one frame passed",
    );
}

/// §3.5 Rules — "active animations continue requesting frames".
///
/// A 500 ms animation with no external frame requests at all keeps the scheduler
/// armed for its whole duration, then retires it.
#[test]
fn an_active_animation_arms_the_scheduler_and_then_retires_it() {
    let (mut rt, sheet, _node) = started(0.5, 0.0, 100.0);
    let sheets = std::slice::from_ref(&sheet);
    let v = scene();

    // The t = 0 present; from here on NOTHING external requests a frame.
    let _ = drain_frames(&mut rt, &v, sheets);

    // 0 .. 500 ms at a 16 ms vsync: 31 ticks reaches 496 ms.
    let mut during = 0usize;
    for _ in 0..31 {
        rt.advance_motion(VSYNC);
        during += drain_frames(&mut rt, &v, sheets);
    }
    assert!(
        during >= 30,
        "an active animation must keep re-arming the scheduler: expected >= 30 \
         frames in 0..500 ms, got {during}"
    );

    // 500 .. 1500 ms: at most the one settle frame at/after t = 500, then nothing.
    let mut after = 0usize;
    for _ in 0..63 {
        rt.advance_motion(VSYNC);
        after += drain_frames(&mut rt, &v, sheets);
    }
    assert!(
        after <= 1,
        "a finished animation must retire its frame request: expected <= 1 frame \
         in 500..1500 ms, got {after}"
    );
}

/// §3.3 Runtime adapters provide — "`Clock`".
///
/// With the injected clock pinned at t=0, 60 presented frames must all sample
/// the same value: no subsystem may read a second time source.
#[test]
fn sixty_frames_on_a_frozen_clock_sample_one_identical_value() {
    let (mut rt, sheet, _node) = started(1.0, 0.0, 100.0);
    let sheets = std::slice::from_ref(&sheet);
    let v = scene();

    let mut widths = Vec::with_capacity(60);
    for _ in 0..60 {
        // The clock is never advanced — `advance_motion` is the only thing that
        // moves this runtime's time, and it is deliberately not called.
        let _ = rt.render_now(&v, sheets);
        widths.push(geom(&rt, at(&rt, &[0, 0])).2);
    }

    assert_eq!(widths.len(), 60);
    for (n, w) in widths.iter().enumerate() {
        approx(
            *w,
            widths[0],
            &format!("frame {n} sampled a different value than frame 0"),
        );
    }
    approx(widths[0], 0.0, "and that value is progress 0.0");
}
