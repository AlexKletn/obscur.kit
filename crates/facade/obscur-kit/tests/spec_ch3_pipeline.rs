//! **SPECIFICATION-DERIVED** frame-pipeline cases for obscur.kit §3.4 ("The
//! conceptual pipeline"), §3.2 ("commit"), and §3.4's determinism requirement.
//!
//! Every expected value below was produced by an agent reading obscur.kit §3
//! with **no access to this repository's source**. A failure here is a defect
//! report about the runtime, and must not be "corrected" by editing the expected
//! value.
#![forbid(unsafe_code)]

#[path = "spec_ch3_support/mod.rs"]
mod support;

use std::cell::Cell;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use obscur_kit::bootstrap::CssPipeline;
use obscur_kit::css::source::CssSource;
use obscur_kit::geometry::{Point, Size};
use obscur_kit::identity::{ComponentId, InternedSymbol, RuntimeId, VersionStamp};
use obscur_kit::interaction::pointer::{PointerDevice, PointerEvent, PointerEventKind, PointerId};
use obscur_kit::motion::track::{Keyframe, KeyframeTrack};
use obscur_kit::motion::transition::TransitionInterruption;
use obscur_kit::motion::value::MotionValue;
use obscur_kit::resources::handle::ResourceKind;
use obscur_kit::resources::port::{DecodedResource, LoadedBytes, ResourceDecoder, ResourceLoader};
use obscur_kit::resources::source::ResourceSource;
use obscur_kit::resources::state::ResourceError;
use obscur_kit::runtime::InputEvent;
use obscur_kit::runtime::UiRuntime;
use obscur_kit::runtime::orchestration::AnimatedKind;
use obscur_kit::runtime::transaction::{
    TransactionCommit, TransactionValidator, UiOperation, UiTransaction,
};
use obscur_kit::style::property::PropertyId;
use obscur_kit::style::selector::PseudoState;
use obscur_kit::view::view::{
    CLASS, EVENT_CLICK, View, ViewEventBinding, ViewFragment, ViewNode, ViewProperty, ViewValue,
};
use support::{approx, at, drain_frames, el, geom, runtime, transforms, view};

const CMD_ACTIVATE: InternedSymbol = InternedSymbol::from_raw(0x0003_0005);

fn commit(ops: Vec<UiOperation>, version: u64) -> TransactionCommit {
    let mut tx = UiTransaction::new();
    for op in ops {
        tx.push(op);
    }
    TransactionValidator::new()
        .validate(&tx)
        .expect("the batch is structurally well-formed");
    TransactionCommit::new(tx.into_operations(), VersionStamp::from_raw(version))
}

fn mouse(kind: PointerEventKind, x: f32, y: f32) -> InputEvent {
    // A §3.4 pipeline stimulus models a REAL user click driving the frame — its
    // concern is timing (input→commit→paint in one frame), not the trust
    // boundary. A real gesture is trusted, and only a trusted click runs its
    // gated default action (§2.3 step 4, enforced in `handle_pointer`), so the
    // event is marked as the user gesture it stands in for. No assertion here
    // depends on trust; the click still activates the handler exactly as a real
    // press would. (Owner-authorized stimulus edit; expected values unchanged.)
    InputEvent::Pointer(
        PointerEvent::new(
            PointerId::MOUSE,
            PointerDevice::Mouse,
            kind,
            Point::new(x, y),
        )
        .as_trusted(),
    )
}

/// §3.4 Conceptual pipeline — "3. dispatch input/events;" then "4. commit
/// state/tree transaction;" then "6. resolve styles;".
///
/// A click handler that commits `class active` must reach paint in the **same**
/// frame that dispatched the click: one frame, red text.
#[test]
fn a_click_handlers_commit_is_painted_by_the_frame_that_dispatched_it() {
    const CSS: &str = ".target { width: 100px; height: 50px; color: rgb(0, 0, 255) } \
                       .active { color: rgb(255, 0, 0) }";

    let mut rt = runtime(200.0, 200.0);
    let active = Rc::new(Cell::new(false));

    {
        let a = active.clone();
        rt.on_command(CMD_ACTIVATE, move || a.set(true));
    }
    {
        let a = active.clone();
        rt.set_root_view(move || {
            let class = if a.get() { "target active" } else { "target" };
            View::single(
                ViewNode::element(ComponentId::from_raw(1))
                    .with_prop(ViewProperty::new(CLASS, ViewValue::Text(class.into())))
                    .with_event(ViewEventBinding::new(EVENT_CLICK, CMD_ACTIVATE)),
            )
        });
    }
    rt.set_stylesheets(vec![support::compile(CSS)]);

    rt.request_frame(obscur_kit::runtime::frame::FrameReason::UiCommit);
    rt.run_due_frame_reactive().expect("the first frame is due");
    let node = at(&rt, &[0]);
    assert_eq!(
        support::color_of(&rt, node).map(|c| (c.r, c.g, c.b)),
        Some((0.0, 0.0, 1.0)),
        "the base rule paints blue before the click"
    );

    // ONE synthetic click, then exactly ONE frame.
    rt.queue_input(mouse(PointerEventKind::Down, 50.0, 25.0));
    rt.queue_input(mouse(PointerEventKind::Up, 50.0, 25.0));
    rt.run_due_frame_reactive()
        .expect("the queued click made a frame due");

    let node = at(&rt, &[0]);
    assert_eq!(
        support::color_of(&rt, node).map(|c| (c.r, c.g, c.b)),
        Some((1.0, 0.0, 0.0)),
        "the handler's commit is resolved by the SAME frame that dispatched the click, \
         not the one after it"
    );
}

/// §3.4 Conceptual pipeline — "7. sample pre-layout motion;" then "8. perform
/// incremental layout;".
///
/// A `width` animation halfway through must be 200px **in that frame's layout**,
/// with the next sibling in normal flow starting at x = 200.
#[test]
fn a_width_animation_reaches_layout_in_the_same_frame_it_is_sampled() {
    const CSS: &str = ".row { display: flex; width: 600px; height: 40px } \
                       .a { width: 100px; height: 20px } \
                       .b { width: 50px; height: 20px }";

    let sheet = support::compile(CSS);
    let sheets = std::slice::from_ref(&sheet);
    let v = view(vec![el(
        1,
        "row",
        vec![el(2, "a", vec![]), el(3, "b", vec![])],
    )]);
    let mut rt = runtime(600.0, 200.0);

    let _ = rt.render_now(&v, sheets);
    let a = at(&rt, &[0, 0]);

    rt.animate(
        a,
        PropertyId::Width,
        KeyframeTrack::from_keyframes(vec![
            Keyframe::new(0.0, MotionValue::scalar(100.0)),
            Keyframe::new(1.0, MotionValue::scalar(300.0)),
        ]),
        1.0,
        AnimatedKind::LengthPx,
        TransitionInterruption::Blend,
    );

    // t = 500 ms.
    rt.advance_motion(0.5);
    let _ = rt.render_now(&v, sheets);

    let a = at(&rt, &[0, 0]);
    let b = at(&rt, &[0, 1]);
    approx(
        geom(&rt, a).2,
        200.0,
        "the animated element's laid-out width",
    );
    approx(geom(&rt, b).0, 200.0, "the next sibling's laid-out x");
}

/// §3.4 Conceptual pipeline — "9. resolve post-layout motion;" (with §4.17
/// "pre-layout and post-layout phases").
///
/// A `transform` animation moves the scene, never the layout, and does not
/// re-run layout.
#[test]
fn a_transform_animation_moves_paint_without_re_running_layout() {
    const CSS: &str = ".box { width: 100px; height: 100px; background-color: #ff0000; \
                       transform: translateX(0px); transition: transform 1s linear } \
                       .box:hover { transform: translateX(100px) }";

    let mut rt = UiRuntime::new(RuntimeId::from_raw(1), Size::new(400.0, 400.0));
    let mut css = CssPipeline::new();
    css.install(&mut rt, &[CssSource::anonymous(CSS)]);
    let sheets = css.sheets().to_vec();
    let v = view(vec![el(1, "box", vec![])]);

    let _ = rt.render_now(&v, &sheets);
    let node = at(&rt, &[0]);

    // Trigger: the style change that starts the transition.
    rt.set_pseudo_state(node, PseudoState::HOVER);

    // t = 0.
    let at_zero = rt.render_now(&v, &sheets);
    let _ = at_zero;
    let (x0, _, w0, _) = geom(&rt, node);
    approx(x0, 0.0, "layout x at t=0");
    approx(w0, 100.0, "layout width at t=0");
    approx(translate_x(&rt), 0.0, "scene translateX at t=0");

    // t = 500 ms.
    rt.advance_motion(0.5);
    let mid = rt.render_now(&v, &sheets);
    let (x1, _, w1, _) = geom(&rt, node);
    approx(x1, 0.0, "layout x at t=500 is UNCHANGED");
    approx(w1, 100.0, "layout width at t=500 is UNCHANGED");
    approx(translate_x(&rt), 50.0, "scene translateX at t=500");
    assert!(
        !mid.relaid_out(),
        "a transform animation must not re-run layout — that is the whole point of \
         classifying it as post-layout motion (§4.3 incremental layout)"
    );

    // t = 1000 ms.
    rt.advance_motion(0.5);
    let _ = rt.render_now(&v, &sheets);
    let (x2, _, w2, _) = geom(&rt, node);
    approx(x2, 0.0, "layout x at t=1000 is UNCHANGED");
    approx(w2, 100.0, "layout width at t=1000 is UNCHANGED");
    approx(translate_x(&rt), 100.0, "scene translateX at t=1000");
}

/// The x-translation of the last frame's outermost non-identity transform.
fn translate_x(rt: &UiRuntime) -> f32 {
    transforms(rt)
        .iter()
        .map(|t| t.apply(Point::new(0.0, 0.0)).x)
        .find(|x| x.abs() > f32::EPSILON)
        .unwrap_or(0.0)
}

/// §3.4 Conceptual pipeline — "4. commit state/tree transaction;" then
/// "5. classify invalidation;" (with §4.3 "typed invalidation reasons").
///
/// A colour-only commit is a **paint** invalidation: no relayout, byte-identical
/// geometry, new colour painted.
#[test]
fn a_colour_only_commit_does_not_re_run_layout() {
    // Translation note: `UiOperation` carries no values, so "the transaction that
    // changes only `color`" is expressed as the style change plus the commit that
    // classifies it. The colour change is driven by a pseudo-state so that nothing
    // about the *tree* changes — only the leaf's computed `color`.
    const CSS: &str = ".page { width: 200px; height: 200px } \
                       .leaf { width: 40px; height: 20px; color: rgb(0, 0, 255) } \
                       .leaf:hover { color: rgb(255, 0, 0) }";

    let sheet = support::compile(CSS);
    let sheets = std::slice::from_ref(&sheet);
    let v = view(vec![el(1, "page", vec![el(2, "leaf", vec![])])]);

    let mut rt = runtime(200.0, 200.0);
    let _ = rt.render_now(&v, sheets);
    let _ = drain_frames(&mut rt, &v, sheets);

    let leaf = at(&rt, &[0, 0]);
    let geometry_before = geom(&rt, leaf);

    rt.set_pseudo_state(leaf, obscur_kit::style::selector::PseudoState::HOVER);
    // The commit says exactly what changed: this node's style, nothing else.
    let decision = rt.apply_transaction(&commit(
        vec![UiOperation::InvalidateStyle { node: leaf }],
        1,
    ));
    assert!(decision.is_some(), "the commit was accepted");

    let frame = rt
        .run_due_frame(&v, sheets)
        .expect("the commit made a frame due");

    assert!(
        frame.restyled(),
        "the frame is still presented and restyled"
    );
    assert!(
        !frame.relaid_out(),
        "a colour-only change is a PAINT invalidation: the layout generation must \
         not advance"
    );
    let leaf = at(&rt, &[0, 0]);
    assert_eq!(
        geom(&rt, leaf),
        geometry_before,
        "the leaf's box geometry is byte-identical"
    );
    assert_eq!(
        support::color_of(&rt, leaf).map(|c| (c.r, c.g, c.b)),
        Some((1.0, 0.0, 0.0)),
        "and the painted colour is the new one"
    );
}

/// §3.4 — "The synchronous frame phase must stay short and deterministic."
///
/// Two identical scripted sessions in one process must produce byte-identical
/// snapshots frame for frame, for all 100 frames.
#[test]
fn two_identical_scripted_sessions_produce_identical_frames() {
    const CSS: &str = ".page { width: 300px; height: 300px } \
                       .a { width: 60px; height: 30px; background-color: #ff0000 } \
                       .b { width: 40px; height: 30px; background-color: #00ff00 } \
                       .c { width: 20px; height: 30px; background-color: #0000ff }";

    fn scripted_run() -> Vec<String> {
        let sheet = support::compile(CSS);
        let sheets = std::slice::from_ref(&sheet);
        let v = view(vec![el(
            1,
            "page",
            vec![el(2, "a", vec![]), el(3, "b", vec![]), el(4, "c", vec![])],
        )]);
        let mut rt = runtime(300.0, 300.0);
        let _ = rt.render_now(&v, sheets);

        let mut fingerprints = Vec::with_capacity(100);
        for frame in 0..100 {
            rt.queue_input(mouse(
                PointerEventKind::Move,
                10.0,
                5.0 + (frame % 3) as f32 * 30.0,
            ));
            rt.advance_motion(support::VSYNC);
            let _ = drain_frames(&mut rt, &v, sheets);
            fingerprints.push(support::frame_fingerprint(&rt));
        }
        fingerprints
    }

    let first = scripted_run();
    let second = scripted_run();

    assert_eq!(first.len(), 100);
    for (n, (a, b)) in first.iter().zip(second.iter()).enumerate() {
        assert_eq!(a, b, "frame {n} differs between two identical runs");
    }
}

/// §3.4 — "The synchronous frame phase must stay short" (with §5.15 "async
/// load/decode/compile", §5.21 "placeholders and fallback first").
///
/// A custom resource loader must never be invoked while the synchronous frame
/// phase is in progress.
#[test]
fn a_custom_loader_is_never_invoked_inside_the_frame_phase() {
    const CSS: &str = ".page { width: 200px; height: 200px; background-image: url(\"probe.png\") }";

    // The recorder is shared with the loader, which must be `Send + Sync`, so it
    // counts through atomics only.
    #[derive(Debug, Default)]
    struct Recorder {
        in_frame: AtomicBool,
        inside: AtomicUsize,
        total: AtomicUsize,
    }

    #[derive(Debug)]
    struct RecordingLoader(Arc<Recorder>);

    impl ResourceLoader for RecordingLoader {
        fn can_load(&self, _source: &ResourceSource) -> bool {
            true
        }
        fn load(&self, _source: &ResourceSource) -> Result<LoadedBytes, ResourceError> {
            self.0.total.fetch_add(1, Ordering::SeqCst);
            if self.0.in_frame.load(Ordering::SeqCst) {
                self.0.inside.fetch_add(1, Ordering::SeqCst);
            }
            Err(ResourceError::SourceUnavailable("probe".into()))
        }
    }

    #[derive(Debug)]
    struct NeverDecoder;

    impl ResourceDecoder for NeverDecoder {
        fn supports(&self, _kind: ResourceKind) -> bool {
            true
        }
        fn decode(
            &self,
            _kind: ResourceKind,
            _bytes: &[u8],
        ) -> Result<DecodedResource, ResourceError> {
            Err(ResourceError::Unsupported("probe".into()))
        }
    }

    let recorder = Arc::new(Recorder::default());
    let sheet = support::compile(CSS);
    let sheets = std::slice::from_ref(&sheet);
    let v = view(vec![el(1, "page", vec![])]);

    let mut rt = runtime(200.0, 200.0);
    rt.set_resource_providers(
        Arc::new(RecordingLoader(recorder.clone())),
        Arc::new(NeverDecoder),
    );

    for _ in 0..3 {
        recorder.in_frame.store(true, Ordering::SeqCst);
        let _ = rt.render_now(&v, sheets);
        recorder.in_frame.store(false, Ordering::SeqCst);
        let _ = rt.poll_resource_tasks();
    }

    assert!(
        recorder.total.load(Ordering::SeqCst) > 0,
        "the loader really was reached for the `url()` background — otherwise this \
         case proves nothing"
    );
    assert_eq!(
        recorder.inside.load(Ordering::SeqCst),
        0,
        "the loader must be invoked 0 times while the synchronous frame phase is in \
         progress; every invocation belongs on a task or between frames"
    );
}

/// Keeps the shared fragment helpers referenced from one place.
#[allow(dead_code)]
fn _fragment_helpers(_: ViewFragment) {}
