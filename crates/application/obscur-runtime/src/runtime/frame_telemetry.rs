#![doc = include_str!("frame_telemetry.doc/_module.md")]

use obscur_diagnostics::{
    BudgetViolation, DiagnosticSubsystem, NullTelemetrySink, PerformanceCounter, PerformanceSpan,
    SpanAttribution, TelemetrySink,
};

use super::FrameBudget;
use crate::frame::FrameId;

#[doc = include_str!("frame_telemetry.doc/FramePhase.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FramePhase {
    #[doc = include_str!("frame_telemetry.doc/Reconcile.md")]
    Reconcile,
    #[doc = include_str!("frame_telemetry.doc/Style.md")]
    Style,
    #[doc = include_str!("frame_telemetry.doc/Layout.md")]
    Layout,
    #[doc = include_str!("frame_telemetry.doc/Compose.md")]
    Compose,
}

impl FramePhase {
    #[doc = include_str!("frame_telemetry.doc/names.md")]
    const fn names(self) -> (&'static str, &'static str, DiagnosticSubsystem) {
        match self {
            Self::Reconcile => ("reconcile", "reconcile.nodes", DiagnosticSubsystem::Tree),
            Self::Style => ("style", "style.resolved", DiagnosticSubsystem::Style),
            Self::Layout => ("layout", "layout.boxes", DiagnosticSubsystem::Layout),
            Self::Compose => ("compose", "compose.commands", DiagnosticSubsystem::Render),
        }
    }

    #[doc = include_str!("frame_telemetry.doc/check.md")]
    fn check(self, budget: FrameBudget, metric: u64) -> Option<BudgetViolation> {
        match self {
            Self::Reconcile => budget.check_reconcile(metric),
            Self::Style => budget.check_style(metric),
            Self::Layout => budget.check_layout(metric),
            Self::Compose => budget.check_compose(metric),
        }
    }
}

#[doc = include_str!("frame_telemetry.doc/FrameTelemetry.md")]
pub(crate) struct FrameTelemetry {
    #[doc = include_str!("frame_telemetry.doc/sink.md")]
    sink: Box<dyn TelemetrySink>,
    #[doc = include_str!("frame_telemetry.doc/budget.md")]
    budget: FrameBudget,
}

impl FrameTelemetry {
    #[doc = include_str!("frame_telemetry.doc/new.md")]
    pub(crate) fn new() -> Self {
        Self {
            sink: Box::new(NullTelemetrySink),
            budget: FrameBudget::unbounded(),
        }
    }

    #[doc = include_str!("frame_telemetry.doc/set_sink.md")]
    pub(crate) fn set_sink(&mut self, sink: Box<dyn TelemetrySink>) {
        self.sink = sink;
    }

    #[doc = include_str!("frame_telemetry.doc/set_budget.md")]
    pub(crate) const fn set_budget(&mut self, budget: FrameBudget) {
        self.budget = budget;
    }

    #[doc = include_str!("frame_telemetry.doc/budget_2.md")]
    pub(crate) const fn budget(&self) -> FrameBudget {
        self.budget
    }

    #[doc = include_str!("frame_telemetry.doc/open_frame.md")]
    pub(crate) fn open_frame(&mut self) {
        self.sink
            .record_span(&PerformanceSpan::new("frame", DiagnosticSubsystem::Runtime));
    }

    #[doc = include_str!("frame_telemetry.doc/close_frame.md")]
    pub(crate) fn close_frame(&mut self, phases_run: u64) {
        self.count(DiagnosticSubsystem::Runtime, "frame.phases_run", phases_run);
    }

    #[doc = include_str!("frame_telemetry.doc/record_phase.md")]
    pub(crate) fn record_phase(&mut self, phase: FramePhase, metric: u64, frame: FrameId) {
        let (name, label, subsystem) = phase.names();
        self.sink.record_span(
            &PerformanceSpan::new(name, subsystem).with_attribution(SpanAttribution::frame(frame)),
        );
        self.count(subsystem, label, metric);
        if let Some(violation) = phase.check(self.budget, metric) {
            self.sink.record_violation(&violation);
        }
    }

    #[doc = include_str!("frame_telemetry.doc/count.md")]
    pub(crate) fn count(&mut self, subsystem: DiagnosticSubsystem, label: &'static str, n: u64) {
        let mut counter = PerformanceCounter::new(subsystem, label);
        counter.add(n);
        self.sink.record_counter(&counter);
    }
}

#[cfg(test)]
mod tests {
    use super::{FramePhase, FrameTelemetry};
    use obscur_diagnostics::{
        BudgetViolation, CollectingTelemetrySink, DiagnosticSubsystem, PerformanceCounter,
        PerformanceSpan, TelemetrySink,
    };
    use std::cell::RefCell;
    use std::rc::Rc;

    use crate::frame::FrameId;
    use crate::runtime::FrameBudget;

    /// A sink shared with the test that installed it: [`FrameTelemetry`] owns its
    /// `Box<dyn TelemetrySink>`, so reading back what was recorded needs a handle
    /// that survives the move.
    #[derive(Clone, Default)]
    struct Shared(Rc<RefCell<CollectingTelemetrySink>>);

    impl TelemetrySink for Shared {
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

    fn channel() -> (FrameTelemetry, Shared) {
        let shared = Shared::default();
        let mut telemetry = FrameTelemetry::new();
        telemetry.set_sink(Box::new(shared.clone()));
        (telemetry, shared)
    }

    const PHASES: [FramePhase; 4] = [
        FramePhase::Reconcile,
        FramePhase::Style,
        FramePhase::Layout,
        FramePhase::Compose,
    ];

    #[test]
    fn a_phases_span_and_counter_name_the_same_subsystem() {
        // The pairing the call sites used to restate by hand: an observer that
        // cannot match a cost counter to the span it was measured under has a
        // number attributed to nothing.
        let (mut telemetry, shared) = channel();
        for phase in PHASES {
            telemetry.record_phase(phase, 7, FrameId::from_raw(3));
        }
        let sink = shared.0.borrow();
        assert_eq!(sink.spans.len(), 4);
        assert_eq!(sink.counters.len(), 4);
        for (span, counter) in sink.spans.iter().zip(&sink.counters) {
            assert_eq!(span.subsystem, counter.subsystem());
            assert_eq!(counter.value(), 7);
            assert_eq!(
                span.attribution.frame,
                Some(FrameId::from_raw(3)),
                "a phase span names the frame it ran in and nothing narrower"
            );
        }
        assert_eq!(
            sink.spans.iter().map(|s| s.name).collect::<Vec<_>>(),
            vec!["reconcile", "style", "layout", "compose"]
        );
        assert_eq!(
            sink.counters
                .iter()
                .map(PerformanceCounter::label)
                .collect::<Vec<_>>(),
            vec![
                "reconcile.nodes",
                "style.resolved",
                "layout.boxes",
                "compose.commands"
            ]
        );
    }

    #[test]
    fn each_phase_is_checked_against_its_own_bound() {
        // The defect this rules out: the compose bound applied after the layout
        // pass. Four bounds, all distinct, and a count that clears three of them
        // and breaks exactly one — so a phase reading the wrong bound reports a
        // violation with the wrong subsystem, or none at all.
        let budget = FrameBudget::unbounded()
            .with_reconcile_nodes(10)
            .with_style_nodes(20)
            .with_layout_boxes(30)
            .with_compose_commands(40);
        let expected = [
            (FramePhase::Reconcile, 11, DiagnosticSubsystem::Tree),
            (FramePhase::Style, 21, DiagnosticSubsystem::Style),
            (FramePhase::Layout, 31, DiagnosticSubsystem::Layout),
            (FramePhase::Compose, 41, DiagnosticSubsystem::Render),
        ];
        for (phase, over, subsystem) in expected {
            let (mut telemetry, shared) = channel();
            telemetry.set_budget(budget);
            // One under its own bound: no phase may complain.
            telemetry.record_phase(phase, over - 1, FrameId::FIRST);
            assert!(
                shared.0.borrow().violations.is_empty(),
                "{phase:?} at {} is within its own bound",
                over - 1
            );
            // One over it: exactly one violation, on this phase's subsystem.
            telemetry.record_phase(phase, over, FrameId::FIRST);
            let sink = shared.0.borrow();
            assert_eq!(
                sink.violations.len(),
                1,
                "{phase:?} exceeded its bound once"
            );
            assert_eq!(sink.violations[0].subsystem(), subsystem);
        }
    }

    #[test]
    fn the_count_that_is_checked_is_the_count_that_is_reported() {
        // One `metric` in, so the violation cannot describe a different amount of
        // work from the counter recorded beside it.
        let (mut telemetry, shared) = channel();
        telemetry.set_budget(FrameBudget::unbounded().with_layout_boxes(2));
        telemetry.record_phase(FramePhase::Layout, 5, FrameId::FIRST);
        let sink = shared.0.borrow();
        assert_eq!(sink.counters[0].value(), 5);
        // The §7.9 cost encoding is one item = one nanosecond.
        assert_eq!(sink.violations[0].measured().as_nanos(), 5);
        assert_eq!(sink.violations[0].budget().as_nanos(), 2);
    }

    #[test]
    fn an_unbounded_budget_records_the_phase_and_no_violation() {
        let (mut telemetry, shared) = channel();
        assert_eq!(telemetry.budget(), FrameBudget::unbounded());
        for phase in PHASES {
            telemetry.record_phase(phase, u64::MAX, FrameId::FIRST);
        }
        let sink = shared.0.borrow();
        assert_eq!(sink.spans.len(), 4, "the phases are still observed");
        assert!(
            sink.violations.is_empty(),
            "an unbounded phase cannot be exceeded, however large the count"
        );
    }

    #[test]
    fn an_idle_frame_is_a_frame_that_ran_no_phases() {
        // The diagnostics-capture shape: the span opens, no phase records, and the
        // tally still lands so the frame is visible as a frame that did nothing.
        let (mut telemetry, shared) = channel();
        telemetry.open_frame();
        telemetry.close_frame(0);
        let sink = shared.0.borrow();
        assert_eq!(
            sink.spans.iter().map(|s| s.name).collect::<Vec<_>>(),
            ["frame"]
        );
        assert_eq!(sink.counters.len(), 1);
        assert_eq!(sink.counters[0].label(), "frame.phases_run");
        assert_eq!(sink.counters[0].value(), 0);
        assert_eq!(sink.counters[0].subsystem(), DiagnosticSubsystem::Runtime);
    }

    #[test]
    fn the_default_channel_records_nowhere_and_installing_a_sink_replays_nothing() {
        // The default is a `NullTelemetrySink`: the records are built and dropped.
        // A sink installed afterwards starts empty — what the null sink swallowed
        // is gone, not buffered.
        let mut telemetry = FrameTelemetry::new();
        telemetry.open_frame();
        telemetry.record_phase(FramePhase::Style, 99, FrameId::FIRST);
        telemetry.close_frame(1);

        let shared = Shared::default();
        telemetry.set_sink(Box::new(shared.clone()));
        assert!(shared.0.borrow().spans.is_empty());
        assert!(shared.0.borrow().counters.is_empty());
    }
}
