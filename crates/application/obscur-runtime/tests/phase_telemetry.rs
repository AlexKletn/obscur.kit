#![doc = include_str!("phase_telemetry.doc/_module.md")]
#![forbid(unsafe_code)]

use std::cell::RefCell;
use std::rc::Rc;

use obscur_diagnostics::{
    BudgetViolation, CollectingTelemetrySink, DiagnosticSeverity, DiagnosticSubsystem,
    PerformanceCounter, PerformanceSpan, TelemetrySink,
};
use obscur_identity::{ComponentId, InternedSymbol};
use obscur_runtime::FrameBudget;
use obscur_runtime::frame::FrameReason;
use obscur_testkit::fixture::{CounterPanel, TestFixture};
use obscur_view::view::{CLASS, View, ViewFragment, ViewNode, ViewProperty, ViewValue};

/// A telemetry sink that shares a [`CollectingTelemetrySink`] behind an
/// `Rc<RefCell<_>>`, so a test can install it into the runtime (which takes
/// ownership of a `Box<dyn TelemetrySink>`) and still read what was recorded
/// afterwards.
#[derive(Clone, Default)]
struct SharedSink(Rc<RefCell<CollectingTelemetrySink>>);

impl TelemetrySink for SharedSink {
    fn record_span(&mut self, span: &PerformanceSpan) {
        self.0.borrow_mut().record_span(span);
    }
    fn record_counter(&mut self, counter: &PerformanceCounter) {
        self.0.borrow_mut().record_counter(counter);
    }
    fn record_violation(&mut self, violation: &BudgetViolation) {
        self.0.borrow_mut().record_violation(violation);
    }
}

#[test]
fn a_frame_records_every_phase_span_and_counter_into_the_sink() {
    let mut runtime = TestFixture::runtime();
    let shared = SharedSink::default();
    runtime.set_telemetry_sink(Box::new(shared.clone()));

    let view = CounterPanel::view();
    let sheet = CounterPanel::sheet();
    let _ = runtime.render_now(&view, std::slice::from_ref(&sheet));

    let sink = shared.0.borrow();
    // Every phase of a full frame recorded its span, in pipeline order.
    let span_names: Vec<&str> = sink.spans.iter().map(|s| s.name).collect();
    assert_eq!(
        span_names,
        vec!["frame", "reconcile", "style", "layout", "compose"]
    );
    // Each phase recorded a cost counter, plus the closing phases_run tally.
    let counter_labels: Vec<&str> = sink
        .counters
        .iter()
        .map(PerformanceCounter::label)
        .collect();
    assert_eq!(
        counter_labels,
        vec![
            "reconcile.nodes",
            "style.resolved",
            "layout.boxes",
            "compose.commands",
            "frame.phases_run",
        ]
    );
    // The full frame ran all four phases and produced real work.
    let phases_run = sink
        .counters
        .iter()
        .find(|c| c.label() == "frame.phases_run")
        .expect("phases_run counter");
    assert_eq!(phases_run.value(), 4);
    assert_eq!(phases_run.subsystem(), DiagnosticSubsystem::Runtime);
    let commands = sink
        .counters
        .iter()
        .find(|c| c.label() == "compose.commands")
        .expect("compose counter");
    assert!(commands.value() > 0, "the matched fill produced a command");
    // No budget configured => no violations.
    assert!(sink.violations.is_empty());
}

#[test]
fn every_phase_span_names_the_frame_it_ran_in() {
    // §7.9 attribution: a span that named only its subsystem was untraceable —
    // you could see that layout ran, but not which frame's layout. Three frames
    // over CHANGING views (so every phase actually runs) must stamp three
    // distinct, increasing frame ids.
    let mut runtime = TestFixture::runtime();
    let shared = SharedSink::default();
    runtime.set_telemetry_sink(Box::new(shared.clone()));

    let sheet = CounterPanel::sheet();
    let sheets = std::slice::from_ref(&sheet);
    for width in [10.0_f32, 20.0, 30.0] {
        let view = View::from_fragment(ViewFragment::from_nodes(vec![
            ViewNode::element(ComponentId::from_raw(10))
                .with_prop(ViewProperty::new(CLASS, ViewValue::Text("counter".into())))
                .with_prop(ViewProperty::new(
                    InternedSymbol::from_raw(0x8000_0055),
                    ViewValue::Float(f64::from(width)),
                )),
        ]));
        let _ = runtime.render_now(&view, sheets);
    }

    let sink = shared.0.borrow();
    let frames: Vec<u64> = sink
        .spans
        .iter()
        .filter(|span| span.name == "layout")
        .map(|span| {
            span.attribution
                .frame
                .expect("a phase span states its frame")
                .raw()
        })
        .collect();
    assert_eq!(frames.len(), 3, "one layout span per frame");
    assert!(
        frames.windows(2).all(|pair| pair[1] > pair[0]),
        "each frame's span names a later frame, got {frames:?}"
    );

    // A phase span is about a FRAME, not a node — leaving `node` empty is the
    // honest answer rather than an invented one.
    let layout = sink
        .spans
        .iter()
        .find(|span| span.name == "layout")
        .expect("a layout span");
    assert_eq!(layout.attribution.node, None);
    assert_eq!(layout.attribution.resource, None);
}

#[test]
fn multiple_frames_accumulate_telemetry() {
    let mut runtime = TestFixture::runtime();
    let shared = SharedSink::default();
    runtime.set_telemetry_sink(Box::new(shared.clone()));

    let view = CounterPanel::view();
    let sheet = CounterPanel::sheet();
    let sheets = std::slice::from_ref(&sheet);
    for _ in 0..3 {
        let _ = runtime.render_now(&view, sheets);
    }

    let sink = shared.0.borrow();
    // Three frames, each opening a "frame" span.
    let frame_spans = sink.spans.iter().filter(|s| s.name == "frame").count();
    assert_eq!(frame_spans, 3);
    // Only the FIRST frame lays out. Frames 2-3 re-render an identical view:
    // reconcile commits empty and no layout-affecting style changed, so
    // `layout_phase` skips the phase and the retained boxes carry over.
    let layout_spans = sink.spans.iter().filter(|s| s.name == "layout").count();
    assert_eq!(layout_spans, 1, "an unchanged re-render must not relayout");
    // 5 spans for the full first frame, 4 for each layout-free repeat.
    assert_eq!(sink.spans.len(), 5 + 2 * 4);
}

#[test]
fn an_exceeded_budget_records_a_violation_that_becomes_a_warning_diagnostic() {
    let mut runtime = TestFixture::runtime();
    let shared = SharedSink::default();
    runtime.set_telemetry_sink(Box::new(shared.clone()));
    // Bound the reconcile phase to zero nodes: any populated tree exceeds it.
    runtime.set_frame_budget(FrameBudget::unbounded().with_reconcile_nodes(0));
    assert_eq!(
        runtime.frame_budget(),
        FrameBudget::unbounded().with_reconcile_nodes(0)
    );

    let view = CounterPanel::view();
    let sheet = CounterPanel::sheet();
    let _ = runtime.render_now(&view, std::slice::from_ref(&sheet));

    let sink = shared.0.borrow();
    assert_eq!(sink.violations.len(), 1, "one reconcile budget violation");
    let violation = &sink.violations[0];
    assert_eq!(violation.subsystem(), DiagnosticSubsystem::Tree);
    // The violation flows into the diagnostics channel as a warning, not a
    // crash (spec §7.9).
    let diagnostic = violation.to_diagnostic();
    assert_eq!(diagnostic.severity, DiagnosticSeverity::Warning);
    assert_eq!(diagnostic.subsystem, DiagnosticSubsystem::Tree);
}

#[test]
fn a_generous_budget_records_no_violation() {
    let mut runtime = TestFixture::runtime();
    let shared = SharedSink::default();
    runtime.set_telemetry_sink(Box::new(shared.clone()));
    runtime.set_frame_budget(
        FrameBudget::unbounded()
            .with_reconcile_nodes(1_000)
            .with_style_nodes(1_000)
            .with_layout_boxes(1_000)
            .with_compose_commands(1_000),
    );

    let view = CounterPanel::view();
    let sheet = CounterPanel::sheet();
    let _ = runtime.render_now(&view, std::slice::from_ref(&sheet));

    assert!(shared.0.borrow().violations.is_empty());
}

#[test]
fn the_default_null_sink_records_nothing_yet_the_frame_still_runs() {
    // No sink installed => the default NullTelemetrySink. The frame must still
    // produce correct output; telemetry is simply discarded (spec §7.9:
    // lightweight counters are opt-in).
    let mut runtime = TestFixture::runtime();
    let view = CounterPanel::view();
    let sheet = CounterPanel::sheet();
    let result = runtime.render_now(&view, std::slice::from_ref(&sheet));
    assert!(result.reconciled());
    assert!(result.is_balanced());
    assert!(!result.snapshot().display_list().is_empty());

    // Swapping in a collecting sink only after the frame proves the default
    // path recorded nowhere observable: the fresh sink is empty until the
    // next frame runs.
    let shared = SharedSink::default();
    runtime.set_telemetry_sink(Box::new(shared.clone()));
    assert!(shared.0.borrow().spans.is_empty());
    assert!(shared.0.borrow().counters.is_empty());
}

#[test]
fn an_idle_frame_records_a_zero_phase_frame() {
    // A diagnostics-capture (idle) frame runs no phases but still marks the
    // frame with a phases_run == 0 counter.
    let mut runtime = TestFixture::runtime();
    let view = CounterPanel::view();
    let sheet = CounterPanel::sheet();
    let sheets = std::slice::from_ref(&sheet);
    // First a full frame so the tree exists (an idle frame requires a
    // populated tree; the first frame is always full).
    let _ = runtime.render_now(&view, sheets);

    let shared = SharedSink::default();
    runtime.set_telemetry_sink(Box::new(shared.clone()));
    runtime.request_frame(FrameReason::DiagnosticsCapture);
    let idle = runtime
        .run_due_frame(&view, sheets)
        .expect("a requested capture is due");
    assert!(!idle.reconciled() && !idle.restyled() && !idle.relaid_out());

    let sink = shared.0.borrow();
    let span_names: Vec<&str> = sink.spans.iter().map(|s| s.name).collect();
    assert_eq!(span_names, vec!["frame"], "idle frame opens only the span");
    assert_eq!(sink.counters.len(), 1);
    assert_eq!(sink.counters[0].label(), "frame.phases_run");
    assert_eq!(sink.counters[0].value(), 0);
}
