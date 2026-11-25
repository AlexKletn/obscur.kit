#![doc = include_str!("spec_ch3_scheduling.doc/_module.md")]
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

#[doc = include_str!("spec_ch3_scheduling.doc/strip_view.md")]
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

#[doc = include_str!("spec_ch3_scheduling.doc/an_idle_runtime_presents_no_frames_while_the_clock_advances.md")]
#[test]
fn an_idle_runtime_presents_no_frames_while_the_clock_advances() {
    const VSYNC_TICKS_IN_TEN_SECONDS: usize = 625;

    let sheet = support::compile(CSS);
    let sheets = std::slice::from_ref(&sheet);
    let v = static_view();
    let mut rt = runtime(200.0, 400.0);

    // Frame 1: first paint.
    let _ = rt.render_now(&v, sheets);
    // Settle anything the first paint owed, so the counter starts clean.
    let _ = drain_frames(&mut rt, &v, sheets);

    let mut presented = 0usize;
    for _ in 0..VSYNC_TICKS_IN_TEN_SECONDS {
        rt.advance_motion(VSYNC);
        presented += drain_frames(&mut rt, &v, sheets);
    }

    assert_eq!(
        presented, 0,
        "10 000 ms of clock with nothing dirty must present 0 frames"
    );
}

#[doc = include_str!("spec_ch3_scheduling.doc/three_requests_and_two_commits_coalesce_into_one_frame.md")]
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

#[test]
fn a_high_priority_input_frame_also_applies_a_pending_structural_update() {
    let sheet = support::compile(CSS);
    let sheets = std::slice::from_ref(&sheet);
    let mut rt = runtime(200.0, 400.0);
    let _ = rt.render_now(&text_view("before"), sheets);

    rt.request_frame(FrameReason::Input);
    rt.request_frame(FrameReason::ExternalRender);
    assert_eq!(drain_frames(&mut rt, &text_view("after"), sheets), 1);
    assert_eq!(tree_text(&rt).as_deref(), Some("after"));
}

#[test]
fn reactive_diagnostics_capture_reuses_the_retained_scene() {
    use std::{cell::Cell, rc::Rc};

    let mut rt = runtime(200.0, 400.0);
    let builds = Rc::new(Cell::new(0));
    let count = Rc::clone(&builds);
    rt.set_root_view(move || {
        count.set(count.get() + 1);
        static_view()
    });
    rt.request_frame(FrameReason::UiCommit);
    let initial = rt.run_due_frame_reactive().unwrap();
    rt.request_frame(FrameReason::DiagnosticsCapture);
    let capture = rt.run_due_frame_reactive().unwrap();
    assert_eq!(builds.get(), 1);
    assert_eq!(capture.snapshot(), initial.snapshot());
    assert!(!capture.restyled() && !capture.relaid_out());
}

#[doc = include_str!("spec_ch3_scheduling.doc/five_queued_pointer_moves_are_five_dispatches_in_one_frame.md")]
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

#[doc = include_str!("spec_ch3_scheduling.doc/input_is_serviced_before_resource_completion_in_one_frame.md")]
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

#[test]
fn an_idle_poll_services_one_background_unit_without_rendering() {
    let mut rt = runtime(200.0, 400.0);
    rt.request_background();
    rt.request_background();

    assert!(rt.run_due_frame(&static_view(), &[]).is_none());
    assert_eq!(rt.pending_background(), 1);
    assert_eq!(rt.serviced_background(), 1);

    rt.set_root_view(|| panic!("an idle poll must leave the view alone"));
    assert!(rt.run_due_frame_reactive().is_none());
    assert!(rt.run_due_frame_reactive().is_none());
    assert_eq!(rt.pending_background(), 0);
    assert_eq!(rt.serviced_background(), 2);
}

#[test]
fn idle_frame_polls_run_one_background_callback_at_a_time() {
    use std::{cell::RefCell, rc::Rc};

    let mut rt = runtime(200.0, 400.0);
    let completed = Rc::new(RefCell::new(Vec::new()));
    for id in [1, 2] {
        let completed = Rc::clone(&completed);
        rt.enqueue_background(move || completed.borrow_mut().push(id));
    }
    assert!(completed.borrow().is_empty());

    assert!(rt.run_due_frame(&static_view(), &[]).is_none());
    assert_eq!(*completed.borrow(), [1]);
    rt.set_root_view(|| panic!("background callbacks alone do not render"));
    assert!(rt.run_due_frame_reactive().is_none());
    assert_eq!(*completed.borrow(), [1, 2]);
    assert_eq!(rt.pending_background(), 0);
    assert_eq!(rt.serviced_background(), 2);
}

#[test]
fn cancelled_background_callbacks_are_discarded_without_counting_as_serviced() {
    use obscur_kit::task::CancellationSource;
    use std::{cell::Cell, rc::Rc};

    let mut rt = runtime(200.0, 400.0);
    let cancelled = CancellationSource::new();
    rt.enqueue_cancellable_background(cancelled.token(), || panic!("cancelled before polling"));
    cancelled.cancel();
    rt.enqueue_cancellable_background(cancelled.token(), || panic!("cancelled before enqueue"));

    let ran = Rc::new(Cell::new(false));
    let observed = Rc::clone(&ran);
    rt.enqueue_background(move || observed.set(true));
    assert_eq!(rt.pending_background(), 2);

    assert!(rt.run_due_frame(&static_view(), &[]).is_none());
    assert_eq!(rt.pending_background(), 1);
    assert_eq!(rt.serviced_background(), 0);
    assert!(!ran.get());
    assert!(rt.run_due_frame(&static_view(), &[]).is_none());
    assert!(ran.get());
    assert_eq!(rt.serviced_background(), 1);
}

#[test]
fn background_deadlines_use_logical_time_and_include_the_boundary() {
    use std::{cell::Cell, rc::Rc, time::Duration};

    let mut rt = runtime(200.0, 400.0);
    rt.enqueue_background_with_deadline(Duration::ZERO, || panic!("zero lifetime"));
    rt.enqueue_background_with_deadline(Duration::from_millis(125), || panic!("expired job"));
    let ran = Rc::new(Cell::new(false));
    let observed = Rc::clone(&ran);
    rt.enqueue_background_with_deadline(Duration::from_millis(250), move || observed.set(true));
    assert_eq!(rt.pending_background(), 2);

    rt.advance_motion(0.125);
    assert!(rt.run_due_frame(&static_view(), &[]).is_none());
    assert_eq!(rt.serviced_background(), 0);
    assert_eq!(rt.pending_background(), 1);
    assert!(rt.run_due_frame(&static_view(), &[]).is_none());
    assert!(ran.get());
    assert_eq!(rt.serviced_background(), 1);
}

#[test]
fn the_host_can_budget_idle_and_fairness_work_separately() {
    use obscur_kit::runtime::{BackgroundBudget, scheduler::StarvationGuard};

    let mut rt = runtime(200.0, 400.0);
    rt.set_background_budget(BackgroundBudget {
        idle_units: 0,
        fairness_units: 2,
    });
    for _ in 0..5 {
        rt.request_background();
    }
    let view = static_view();
    assert!(rt.run_due_frame(&view, &[]).is_none());
    assert_eq!(rt.pending_background(), 5);
    for _ in 0..StarvationGuard::DEFAULT_THRESHOLD {
        rt.request_frame(FrameReason::Input);
        assert!(rt.run_due_frame(&view, &[]).is_some());
    }
    assert_eq!(rt.pending_background(), 5);
    rt.request_frame(FrameReason::Input);
    assert!(rt.run_due_frame(&view, &[]).is_some());
    assert_eq!(rt.pending_background(), 3);

    rt.set_background_budget(BackgroundBudget {
        idle_units: 2,
        fairness_units: 0,
    });
    assert!(rt.run_due_frame(&view, &[]).is_none());
    assert_eq!(rt.pending_background(), 1);
    assert_eq!(rt.serviced_background(), 4);
}

#[doc = include_str!("spec_ch3_scheduling.doc/background_work_completes_under_sustained_input_saturation.md")]
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

#[doc = include_str!("spec_ch3_scheduling.doc/three_pending_reasons_produce_exactly_one_frame.md")]
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

#[doc = include_str!("spec_ch3_scheduling.doc/_sheet_type_is_used.md")]
#[allow(dead_code)]
fn _sheet_type_is_used(_: &Stylesheet) {}
