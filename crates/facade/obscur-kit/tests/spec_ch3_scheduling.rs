//! **SPECIFICATION-DERIVED** scheduler cases for obscur.kit §3.5 ("Rules") and
//! §3.6 ("Priority order", "starvation prevention for background work").
//!
//! Every expected value below was produced by an agent reading obscur.kit §3
//! with **no access to this repository's source**. A failure here is a defect
//! report about the runtime, and must not be "corrected" by editing the expected
//! value.
#![forbid(unsafe_code)]

#[path = "spec_ch3_support/mod.rs"]
mod support;

use obscur_kit::geometry::Point;
use obscur_kit::identity::{NodeId, VersionStamp};
use obscur_kit::interaction::pointer::{PointerDevice, PointerEvent, PointerEventKind, PointerId};
use obscur_kit::runtime::InputEvent;
use obscur_kit::runtime::frame::FrameReason;
use obscur_kit::runtime::transaction::{
    TransactionCommit, TransactionValidator, UiOperation, UiTransaction,
};
use obscur_kit::style::matching::Stylesheet;
use obscur_kit::view::view::View;
use support::{VSYNC, at, drain_frames, el, runtime, tree_text, txt, view};

const CSS: &str = ".page { width: 200px; height: 400px } \
                   .row { width: 100px; height: 12px; background-color: #ff0000 }";

fn static_view() -> View {
    view(vec![el(1, "page", vec![el(2, "row", vec![])])])
}

/// Five stacked 12px rows: row *i* covers y `12i .. 12i+12`.
fn strip_view() -> View {
    view(vec![el(
        1,
        "page",
        (0..5).map(|i| el(10 + i, "row", vec![])).collect(),
    )])
}

fn text_view(content: &str) -> View {
    view(vec![el(1, "page", vec![el(2, "row", vec![txt(content)])])])
}

fn moved_to(y: f32) -> InputEvent {
    InputEvent::Pointer(PointerEvent::new(
        PointerId::from_raw(1),
        PointerDevice::Mouse,
        PointerEventKind::Move,
        Point::new(5.0, y),
    ))
}

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

/// §3.5 Rules — "no busy loop".
///
/// After the first frame is presented, advancing the deterministic clock by
/// 10 000 ms in 16 ms steps with no input, commit or resource event must present
/// **zero** further frames.
#[test]
fn an_idle_runtime_presents_no_frames_while_the_clock_advances() {
    let sheet = support::compile(CSS);
    let sheets = std::slice::from_ref(&sheet);
    let v = static_view();
    let mut rt = runtime(200.0, 400.0);

    // Frame 1: first paint.
    let _ = rt.render_now(&v, sheets);
    // Settle anything the first paint owed, so the counter starts clean.
    let _ = drain_frames(&mut rt, &v, sheets);

    let mut presented = 0usize;
    for _ in 0..625 {
        // 625 * 16 ms = 10 000 ms.
        rt.advance_motion(VSYNC);
        presented += drain_frames(&mut rt, &v, sheets);
    }

    assert_eq!(
        presented, 0,
        "10 000 ms of clock with nothing dirty must present 0 frames"
    );
}

/// §3.5 Rules — "updates are coalesced".
///
/// Three external frame requests and two committed transactions between two
/// frame boundaries must produce **exactly one** frame, and that frame must show
/// the *later* transaction's state.
#[test]
fn three_requests_and_two_commits_coalesce_into_one_frame() {
    let sheet = support::compile(CSS);
    let sheets = std::slice::from_ref(&sheet);
    let mut rt = runtime(200.0, 400.0);

    let _ = rt.render_now(&text_view("a"), sheets);
    let _ = drain_frames(&mut rt, &text_view("a"), sheets);

    rt.request_frame(FrameReason::ExternalRender);
    rt.request_frame(FrameReason::ExternalRender);
    rt.request_frame(FrameReason::ExternalRender);

    let node = at(&rt, &[0, 0]);
    let a = rt.apply_transaction(&commit(vec![UiOperation::Rerender { node }], 1));
    let b = rt.apply_transaction(&commit(vec![UiOperation::ReplaceRoot], 2));
    assert!(a.is_some(), "transaction A was recorded as applied");
    assert!(b.is_some(), "transaction B was recorded as applied");

    let later = text_view("b");
    let presented = drain_frames(&mut rt, &later, sheets);

    assert_eq!(
        presented, 1,
        "3 requests + 2 commits between frame boundaries is exactly 1 frame"
    );
    assert_eq!(
        tree_text(&rt).as_deref(),
        Some("b"),
        "the coalesced frame shows the LATER transaction's text"
    );
}

/// §3.5 Rules — "stale intermediate frames may be skipped".
///
/// The clause permits skipping *frames*, never *input*. Five queued pointer
/// moves are dispatched as five events by one frame.
#[test]
fn five_queued_pointer_moves_are_five_dispatches_in_one_frame() {
    let sheet = support::compile(CSS);
    let sheets = std::slice::from_ref(&sheet);
    let v = strip_view();
    let mut rt = runtime(200.0, 400.0);

    let _ = rt.render_now(&v, sheets);
    let _ = drain_frames(&mut rt, &v, sheets);

    let rows: Vec<NodeId> = (0..5).map(|i| at(&rt, &[0, i])).collect();

    // The deriver's x = 10,20,30,40,50 becomes y = 10,20,30,40,50 down a column
    // of 12px rows, because normal flow stacks blocks vertically. Row i covers
    // 12i..12i+12, so the five positions land on rows 0,1,2,3,4 in order.
    for y in [10.0, 20.0, 30.0, 40.0, 50.0] {
        rt.queue_input(moved_to(y));
    }
    assert_eq!(
        rt.pending_input(),
        5,
        "five events are queued, none dropped"
    );

    let presented = drain_frames(&mut rt, &v, sheets);

    let dispatched: Vec<Option<NodeId>> = rt
        .take_input_dispatches()
        .iter()
        .map(|d| match d {
            obscur_kit::runtime::InputDispatch::Pointer(p) => p.target,
            other => panic!("expected pointer dispatches, got {other:?}"),
        })
        .collect();

    let expected: Vec<Option<NodeId>> = rows.iter().map(|&n| Some(n)).collect();
    assert_eq!(
        dispatched, expected,
        "the handler log is exactly the five points, in order, with no coalescing of input"
    );
    assert_eq!(presented, 1, "and the five events cost exactly one frame");
    assert_eq!(
        rt.hovered(),
        Some(rows[4]),
        "the pointer position rendered in that frame is the LAST one"
    );
}

/// §3.6 Priority order — "1. input, focus, pointer and IME; ... 4. resource
/// completion;".
///
/// With a key/pointer event and a resource completion both pending in one tick,
/// the log is `["input", "resource"]` — input first — in that single frame.
#[test]
fn input_is_serviced_before_resource_completion_in_one_frame() {
    let sheet = support::compile(CSS);
    let sheets = std::slice::from_ref(&sheet);
    let v = strip_view();
    let mut rt = runtime(200.0, 400.0);

    let _ = rt.render_now(&v, sheets);
    let _ = drain_frames(&mut rt, &v, sheets);

    // Both pending in the same tick, resource first in arrival order so a FIFO
    // queue would serve it first.
    rt.request_frame(FrameReason::ResourceCompletion);
    rt.queue_input(moved_to(10.0));

    let first = rt
        .run_due_frame(&v, sheets)
        .expect("a frame is due with both reasons pending");
    let _ = first;

    assert_eq!(
        rt.take_input_dispatches().len(),
        1,
        "input is dispatched by the FIRST frame drained: priority order, not arrival order"
    );

    let remaining = drain_frames(&mut rt, &v, sheets);
    assert_eq!(
        remaining, 0,
        "both reasons are served by that single frame — 1 frame total, not 2"
    );
}

/// §3.6 The scheduler supports — "starvation prevention for background work".
///
/// Under 300 consecutive frames of input saturation, queued background work must
/// still complete. The spec gives no bound, so the discriminator is
/// finite-vs-never.
#[test]
fn background_work_completes_under_sustained_input_saturation() {
    let sheet = support::compile(CSS);
    let sheets = std::slice::from_ref(&sheet);
    let v = strip_view();
    let mut rt = runtime(200.0, 400.0);

    let _ = rt.render_now(&v, sheets);
    let _ = drain_frames(&mut rt, &v, sheets);

    // A background text-shaping job and a background image decode.
    rt.request_background();
    rt.request_background();
    assert_eq!(rt.pending_background(), 2);

    let mut completed_at: Option<usize> = None;
    for frame in 0..300 {
        rt.queue_input(moved_to(10.0 + (frame % 5) as f32 * 12.0));
        let _ = drain_frames(&mut rt, &v, sheets);
        if rt.pending_background() == 0 && completed_at.is_none() {
            completed_at = Some(frame);
        }
    }

    assert_eq!(
        rt.serviced_background(),
        2,
        "both background jobs completed within the 300 saturated frames"
    );
    assert!(
        completed_at.is_some(),
        "the completion frame index is finite — strict priority would leave it never"
    );
}

/// §3.4 Conceptual pipeline — "2. choose frame reasons;" (with §3.5 "Frame
/// reasons include: input; UI commit; animation; ...").
///
/// Three reasons pending at once is **one** frame, not three.
#[test]
fn three_pending_reasons_produce_exactly_one_frame() {
    let sheet = support::compile(CSS);
    let sheets = std::slice::from_ref(&sheet);
    let v = strip_view();
    let mut rt = runtime(200.0, 400.0);

    let _ = rt.render_now(&v, sheets);
    let _ = drain_frames(&mut rt, &v, sheets);

    rt.queue_input(moved_to(10.0));
    let node = at(&rt, &[0, 0]);
    let _ = rt.apply_transaction(&commit(vec![UiOperation::Rerender { node }], 1));
    rt.request_frame(FrameReason::Animation);

    let presented = drain_frames(&mut rt, &v, sheets);
    assert_eq!(
        presented, 1,
        "input + UI commit + animation pending together is exactly 1 frame"
    );
}

/// Keeps the unused-import lint honest about the shared `Stylesheet` alias.
#[allow(dead_code)]
fn _sheet_type_is_used(_: &Stylesheet) {}
