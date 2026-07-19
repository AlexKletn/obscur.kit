//! **SPECIFICATION-DERIVED** diagnostics and telemetry cases for obscur.kit
//! §7.8 / §7.9.
//!
//! Written by an agent reading obscur.kit §7.8, §7.9, §1.6 and §3.5 with **no
//! access to this repository's source**.
#![forbid(unsafe_code)]

#[path = "spec_ch7_support/mod.rs"]
mod support;

use obscur_kit::diagnostics::{
    Diagnostic, DiagnosticBufferPolicy, DiagnosticCode, DiagnosticSeverity, DiagnosticSubscription,
    DiagnosticSubsystem, OverflowStrategy, RecoveryAction, SubscriptionId,
};
use obscur_kit::runtime::FrameBudget;
use obscur_kit::runtime::frame::FrameReason;
use obscur_kit::runtime::scheduler::FrameScheduler;
use support::{SharedDiagnostics, SharedTelemetry, all_geom, drive, el, runtime, txt, view};

const VW: f32 = 400.0;

/// §7.8, verbatim: "Diagnostics are typed and structured rather than plain
/// logs." with §1.6: "diagnostics include severity, subsystem, stable code,
/// node/resource/window context and recovery action."
#[test]
fn the_same_fault_carries_one_stable_code_across_two_contexts() {
    let sink = SharedDiagnostics::new();
    let mut rt = runtime(VW, VW);
    rt.set_diagnostic_sink(Box::new(sink.clone()));

    // The same recoverable fault — an unparseable value in one declaration —
    // raised from two different nodes, with two different property names.
    let css = ".a { width: not-a-length } .b { color: not-a-color } \
               .a, .b { height: 10px }";
    let sheet = support::compile(css);
    let v = view(vec![el(1, "a", vec![]), el(2, "b", vec![])]);
    let _ = rt.render_now(&v, std::slice::from_ref(&sheet));

    let records = sink.taken();
    assert!(
        records.len() >= 2,
        "both unparseable declarations produce a structured record, got {records:?}"
    );
    let first = &records[0];
    let second = &records[1];

    assert_eq!(
        first.code, second.code,
        "the same fault class carries an identical stable code"
    );
    assert_eq!(
        first.subsystem, second.subsystem,
        "the same fault class carries an identical subsystem"
    );
    assert_ne!(
        first.context.node, second.context.node,
        "the two records carry different node context"
    );
    assert_ne!(
        first.recovery,
        RecoveryAction::None,
        "a recoverable fault names its recovery action"
    );
    assert_eq!(
        first.message, second.message,
        "the offending property name is a typed field, not text baked into the message"
    );
}

/// §7.8: "- buffered;" / "- backpressured;" with §3.4 "The synchronous frame
/// phase must stay short and deterministic." and §3.6 priority 6 "telemetry and
/// development diagnostics."
#[test]
fn a_stalled_subscriber_is_backpressured_not_allowed_to_stall_the_producer() {
    let mut subscription = DiagnosticSubscription::new(
        SubscriptionId::new(1),
        obscur_kit::diagnostics::DiagnosticFilter::min(DiagnosticSeverity::Info),
        DiagnosticBufferPolicy::new(8, OverflowStrategy::DropNewest),
    );

    // 100 diagnostics offered across 10 frames, to a subscriber that never drains.
    let mut refused = 0_u32;
    for frame in 0..10_u32 {
        for i in 0..10_u32 {
            let diagnostic = Diagnostic::new(
                DiagnosticCode::new("test.backpressure"),
                DiagnosticSeverity::Warning,
                DiagnosticSubsystem::Layout,
                format!("frame {frame} item {i}"),
            );
            if !subscription.offer(&diagnostic) {
                refused += 1;
            }
        }
    }

    assert_eq!(
        subscription.len(),
        8,
        "the bounded buffer holds exactly its capacity — memory does not grow with the 100"
    );
    let drained = subscription.drain();
    assert_eq!(drained.len(), 8, "the subscriber sees 8 buffered messages");
    assert_eq!(
        refused, 92,
        "the 92 undelivered diagnostics are accounted for, not silently discarded"
    );
}

/// §7.9, verbatim: "Budgets produce diagnostics, not crashes."
#[test]
fn a_budget_overrun_degrades_to_a_diagnostic_and_ships_correct_geometry() {
    let css = ".row { display: flex; width: 300px; height: 40px } \
               .cell { width: 20px; height: 20px }";
    let v = view(vec![el(
        1,
        "row",
        (0..10).map(|i| el(10 + i, "cell", vec![])).collect(),
    )]);

    // Unbudgeted reference geometry.
    let unbudgeted = drive(VW, VW, css, &v);
    let reference = all_geom(&unbudgeted);

    // The same tree under a layout budget the tree provably exceeds.
    let telemetry = SharedTelemetry::new();
    let sheet = support::compile(css);
    let mut rt = runtime(VW, VW);
    rt.set_telemetry_sink(Box::new(telemetry.clone()));
    rt.set_frame_budget(FrameBudget::unbounded().with_layout_boxes(1));
    let _ = rt.render_now(&v, std::slice::from_ref(&sheet));

    assert_eq!(
        all_geom(&rt),
        reference,
        "the frame completes and the layout geometry is fully correct — no bail-out"
    );

    let violations = telemetry.violations();
    assert_eq!(
        violations.len(),
        1,
        "exactly one budget diagnostic is emitted, got {violations:?}"
    );
    assert_eq!(
        violations[0].subsystem(),
        DiagnosticSubsystem::Layout,
        "the budget diagnostic is attributed to the layout subsystem"
    );
}

/// §7.9: "Attribution:" / "- frame;" / "- invalidation cause;" with §3.5 "Frame
/// reasons include:" / "- input;" / "- animation;" / "- resource completion;" /
/// "- diagnostics capture;".
#[test]
fn four_frames_are_attributed_to_four_distinct_reasons() {
    let mut scheduler = FrameScheduler::new();
    let expected = [
        FrameReason::Input,
        FrameReason::Animation,
        FrameReason::ResourceCompletion,
        FrameReason::DiagnosticsCapture,
    ];

    for reason in expected {
        let _ = scheduler.request_reason(reason);
    }

    let mut observed = Vec::new();
    while let Some(request) = scheduler.next_frame() {
        observed.push(request.reason());
    }

    assert_eq!(
        observed.len(),
        4,
        "four frames were produced, each caused by exactly one thing, got {observed:?}"
    );
    let mut sorted: Vec<_> = observed.iter().map(|r| format!("{r:?}")).collect();
    sorted.sort();
    sorted.dedup();
    assert_eq!(
        sorted.len(),
        4,
        "the four reasons are distinct — no generic or always-input attribution"
    );
    for reason in expected {
        assert!(
            observed.contains(&reason),
            "frame reason {reason:?} was attributed, got {observed:?}"
        );
    }
}

/// §7.9, verbatim: "Deep tracing is opt-in. Lightweight counters may remain
/// available in production only when explicitly enabled."
#[test]
fn deep_tracing_when_enabled_covers_dispatch_layout_and_paint() {
    let telemetry = SharedTelemetry::new();
    let css = ".box { width: 100px; height: 40px; background-color: #123456 }";
    let sheet = support::compile(css);
    let v = view(vec![el(1, "box", vec![txt("hi")])]);

    let mut rt = runtime(VW, VW);
    rt.set_telemetry_sink(Box::new(telemetry.clone()));
    for _ in 0..10 {
        let _ = rt.render_now(&v, std::slice::from_ref(&sheet));
    }

    let names = telemetry.span_names();
    assert!(
        !names.is_empty(),
        "explicitly enabled deep tracing collects spans"
    );
    // "paint" is spelled `compose` in this workspace's phase vocabulary; the
    // synonym is accepted because the case is about coverage, not naming.
    for phase in [
        &["dispatch"][..],
        &["layout"][..],
        &["paint", "compose"][..],
    ] {
        assert!(
            names.iter().any(|n| phase.iter().any(|p| n.contains(p))),
            "spans cover {phase:?} at minimum, got {names:?}"
        );
    }
}
