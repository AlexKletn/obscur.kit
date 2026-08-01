#![doc = include_str!("frame_scheduling.doc/_module.md")]
#![forbid(unsafe_code)]

use obscur_identity::VersionStamp;
use obscur_runtime::frame::{FrameId, FrameReason};
use obscur_runtime::scheduler::StarvationGuard;
use obscur_runtime::transaction::{TransactionCommit, UiOperation};
use obscur_testkit::fixture::{CounterPanel, TestFixture};

/// A transaction's operations, not its reason, decide what the frame it
/// schedules recalculates.
#[test]
fn a_committed_transaction_decides_the_frames_phases() {
    let sheet = CounterPanel::sheet();
    let sheets = std::slice::from_ref(&sheet);

    // Establish the retained tree: the first frame is unconditionally full.
    let mut runtime = TestFixture::runtime();
    assert!(
        runtime
            .render_now(&CounterPanel::view(), sheets)
            .reconciled()
    );
    let node = runtime.tree().root().expect("a root");
    let before = runtime.tree().len();

    // Commit a transaction that invalidates a style and NOTHING else, then run
    // the frame it scheduled — handing it the BIGGER view.
    assert!(
        runtime
            .apply_transaction(&TransactionCommit::new(
                vec![UiOperation::InvalidateStyle { node }],
                VersionStamp::from_raw(9),
            ))
            .is_some(),
        "a non-empty commit requests a frame"
    );
    let result = runtime
        .run_due_frame(&CounterPanel::nested_view(), sheets)
        .expect("the commit scheduled a frame");

    // THE assertion: the operations decided the plan. A `UiCommit` frame
    // classified from its reason alone runs every phase and would have pulled
    // the new node in; this one knows the tree cannot have changed, because no
    // operation said it did — so the extra node is not adopted.
    assert!(
        !result.reconciled(),
        "a style-only transaction does not reconcile"
    );
    assert_eq!(
        runtime.tree().len(),
        before,
        "the tree really was left alone"
    );
    assert!(result.restyled(), "but it does restyle");
    assert!(result.composed());

    // The control: the same view through the ordinary reason-derived path DOES
    // reconcile — so the difference above is the plan, not the view.
    runtime.request_frame(FrameReason::UiCommit);
    let control = runtime
        .run_due_frame(&CounterPanel::nested_view(), sheets)
        .expect("a frame is due");
    assert!(control.reconciled());
    assert!(runtime.tree().len() > before, "and now the node is adopted");
}

/// A transaction that asked for nothing must cost nothing.
#[test]
fn an_empty_commit_schedules_no_frame_at_all() {
    let mut runtime = TestFixture::runtime();
    let view = CounterPanel::view();
    let sheet = CounterPanel::sheet();
    runtime.render_now(&view, std::slice::from_ref(&sheet));

    let empty = TransactionCommit::new(Vec::new(), VersionStamp::from_raw(9));
    assert!(
        runtime.apply_transaction(&empty).is_none(),
        "nothing was asked for, so no frame is requested"
    );
    assert!(
        !runtime.has_pending_frame(),
        "an abandoned transaction must cost zero frames, not one that does nothing"
    );
}

/// One frame serving two commits owes the union of what they asked for.
#[test]
fn two_commits_between_frames_both_get_their_phases() {
    let sheet = CounterPanel::sheet();
    let sheets = std::slice::from_ref(&sheet);
    let mut runtime = TestFixture::runtime();
    runtime.render_now(&CounterPanel::view(), sheets);
    let node = runtime.tree().root().expect("a root");
    let before = runtime.tree().len();

    // A cheap commit, then a structural one, with no frame in between: the
    // single frame that serves both owes the union, so it must reconcile —
    // proven by the new node being adopted, which the cheap plan alone would
    // have skipped (see the test above).
    runtime.apply_transaction(&TransactionCommit::new(
        vec![UiOperation::InvalidateStyle { node }],
        VersionStamp::from_raw(9),
    ));
    runtime.apply_transaction(&TransactionCommit::new(
        vec![UiOperation::Rerender { node }],
        VersionStamp::from_raw(10),
    ));

    let result = runtime
        .run_due_frame(&CounterPanel::nested_view(), sheets)
        .expect("a frame is due");
    assert!(
        result.reconciled(),
        "the structural commit must not be lost to the cheap one"
    );
    assert!(runtime.tree().len() > before);
}

/// The tree mutation a frame performs goes through the atomic commit boundary,
/// and the version stamp is how that is observable.
#[test]
fn tree_change_commits_through_the_transaction_boundary() {
    // S2: the live frame's tree mutation flows through the atomic transaction
    // commit boundary (spec §3.2), observable via the commit version stamp —
    // while the produced frame output stays identical.
    let mut runtime = TestFixture::runtime();
    let view = CounterPanel::view();
    let sheet = CounterPanel::sheet();
    let sheets = std::slice::from_ref(&sheet);

    // No frame has committed yet.
    assert_eq!(runtime.commit_version(), VersionStamp::from_raw(0));

    // First frame populates the tree: the reconcile is committed, so the
    // version stamp advances — proving the mutation went through the commit.
    let first = runtime.render_now(&view, sheets);
    assert!(first.reconciled());
    assert_eq!(runtime.commit_version(), VersionStamp::from_raw(1));

    // A second frame over the same view is an idempotent reconcile: it commits
    // empty and does not advance the version, yet still produces the exact same
    // display list as the first frame (behavior is unchanged by the boundary).
    let second = runtime.render_now(&view, sheets);
    assert_eq!(
        runtime.commit_version(),
        VersionStamp::from_raw(1),
        "an idempotent reconcile does not advance the commit version"
    );
    let first_commands: Vec<_> = first.snapshot().display_list().commands().to_vec();
    let second_commands: Vec<_> = second.snapshot().display_list().commands().to_vec();
    assert_eq!(
        first_commands, second_commands,
        "the committed path yields the same display list as before"
    );
}

/// A frame runs when one was requested, and exactly once per request.
#[test]
fn run_due_frame_is_gated_by_the_scheduler() {
    let mut runtime = TestFixture::runtime();
    let view = CounterPanel::view();
    let sheet = CounterPanel::sheet();
    let sheets = std::slice::from_ref(&sheet);

    // Nothing requested → nothing due.
    assert!(runtime.run_due_frame(&view, sheets).is_none());

    // One request → exactly one frame, then idle again (coalesced).
    runtime.request_frame(FrameReason::Input);
    assert!(runtime.has_pending_frame());
    let produced = runtime
        .run_due_frame(&view, sheets)
        .expect("a requested frame is due");
    assert_eq!(produced.id(), FrameId::FIRST);
    assert!(!runtime.has_pending_frame());
    assert!(runtime.run_due_frame(&view, sheets).is_none());

    // A second request advances the frame id (proves one frame per request).
    runtime.request_frame(FrameReason::Input);
    let second = runtime
        .run_due_frame(&view, sheets)
        .expect("the next request is due");
    assert_eq!(second.id(), FrameId::FIRST.next());
}

/// Two requests naming the same reason are one frame's worth of work.
#[test]
fn same_reason_requests_coalesce_into_one_frame() {
    let mut runtime = TestFixture::runtime();
    let view = CounterPanel::view();

    runtime.request_frame(FrameReason::Animation);
    runtime.request_frame(FrameReason::Animation);
    assert_eq!(runtime.pending_frames(), 1, "same reason coalesces");
    assert!(runtime.run_due_frame(&view, &[]).is_some());
    assert!(runtime.run_due_frame(&view, &[]).is_none());
}

/// Background work is a signal, not a frame.
#[test]
fn requesting_background_marks_it_pending_without_a_frame() {
    let mut runtime = TestFixture::runtime();
    // Background work is a signal, not a frame: it must not enqueue a frame.
    assert_eq!(runtime.pending_background(), 0);
    assert!(!runtime.has_pending_frame());
    runtime.request_background();
    assert_eq!(runtime.pending_background(), 1);
    assert!(
        !runtime.has_pending_frame(),
        "background work is not a frame request"
    );
    assert!(!runtime.is_background_starved());
    // A second unit accumulates but does not re-signal a frame either.
    runtime.request_background();
    assert_eq!(runtime.pending_background(), 2);
    assert!(!runtime.has_pending_frame());
}

/// §3.6 starvation prevention is live on the real drain path, not only on the
/// bare scheduler.
#[test]
fn background_work_is_serviced_through_the_drain_path_despite_higher_priority_frames() {
    // Drive the *runtime* (not the bare scheduler): note background work, then
    // pound it with a stream of higher-priority Input frames. §3.6 starvation
    // prevention must eventually promote and service the background unit
    // through `run_due_frame` — proving `background_starved`/`service_background`
    // are live on the real path.
    let mut runtime = TestFixture::runtime();
    let view = CounterPanel::view();
    let sheet = CounterPanel::sheet();
    let sheets = std::slice::from_ref(&sheet);

    runtime.request_background();
    assert_eq!(runtime.serviced_background(), 0);

    // The default fairness threshold before background work is starved.
    let threshold = StarvationGuard::DEFAULT_THRESHOLD;

    // Serve `threshold` higher-priority (Input) frames while the background
    // unit waits. None of these service the background work yet.
    for _ in 0..threshold {
        runtime.request_frame(FrameReason::Input);
        assert!(runtime.run_due_frame(&view, sheets).is_some());
    }
    assert_eq!(
        runtime.serviced_background(),
        0,
        "background work is not serviced before the fairness threshold"
    );
    assert!(
        runtime.is_background_starved(),
        "after `threshold` preempting frames the background band is starved"
    );

    // The next drain call promotes and services the starved background unit.
    runtime.request_frame(FrameReason::Input);
    assert!(runtime.run_due_frame(&view, sheets).is_some());
    assert_eq!(
        runtime.serviced_background(),
        1,
        "the starved background unit is serviced through the drain path"
    );
    assert_eq!(runtime.pending_background(), 0);
    // Servicing reset fairness and cleared the pending signal.
    assert!(!runtime.is_background_starved());
}

/// Servicing one unit does not clear the band.
#[test]
fn serviced_background_stays_pending_until_every_unit_drains() {
    // Two units: the first promotion services one; the band stays pending and
    // is starved again by another burst, servicing the second.
    let mut runtime = TestFixture::runtime();
    let view = CounterPanel::view();
    let sheet = CounterPanel::sheet();
    let sheets = std::slice::from_ref(&sheet);
    let threshold = StarvationGuard::DEFAULT_THRESHOLD;

    runtime.request_background();
    runtime.request_background();
    assert_eq!(runtime.pending_background(), 2);

    // First starvation cycle services one unit.
    for _ in 0..threshold {
        runtime.request_frame(FrameReason::Input);
        let _ = runtime.run_due_frame(&view, sheets);
    }
    runtime.request_frame(FrameReason::Input);
    let _ = runtime.run_due_frame(&view, sheets);
    assert_eq!(runtime.serviced_background(), 1);
    assert_eq!(runtime.pending_background(), 1);
    assert!(
        !runtime.is_background_starved(),
        "fairness reset, so the remaining unit is not immediately starved"
    );

    // Second starvation cycle services the last unit and clears the band.
    for _ in 0..threshold {
        runtime.request_frame(FrameReason::Input);
        let _ = runtime.run_due_frame(&view, sheets);
    }
    runtime.request_frame(FrameReason::Input);
    let _ = runtime.run_due_frame(&view, sheets);
    assert_eq!(runtime.serviced_background(), 2);
    assert_eq!(runtime.pending_background(), 0);
    assert!(!runtime.is_background_starved());
}
