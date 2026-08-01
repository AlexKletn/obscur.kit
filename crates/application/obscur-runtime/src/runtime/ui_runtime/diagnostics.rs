#![doc = include_str!("diagnostics.doc/_module.md")]

use obscur_diagnostics::{Diagnostic, DiagnosticSink, TelemetrySink};

use crate::frame::FrameReason;
use crate::orchestration::{InspectInfo, TreeInspection};
use crate::runtime::{FrameBudget, TreeInspector};

use super::UiRuntime;

impl UiRuntime {
    #[doc = include_str!("diagnostics.doc/set_diagnostic_sink.md")]
    pub fn set_diagnostic_sink(&mut self, sink: Box<dyn DiagnosticSink>) {
        self.diagnostics = sink;
    }

    #[doc = include_str!("diagnostics.doc/report_diagnostic.md")]
    pub fn report_diagnostic(&self, diagnostic: &Diagnostic) {
        self.diagnostics.emit(diagnostic);
    }

    #[doc = include_str!("diagnostics.doc/set_telemetry_sink.md")]
    pub fn set_telemetry_sink(&mut self, sink: Box<dyn TelemetrySink>) {
        self.telemetry.set_sink(sink);
    }

    #[doc = include_str!("diagnostics.doc/set_frame_budget.md")]
    pub fn set_frame_budget(&mut self, budget: FrameBudget) {
        self.telemetry.set_budget(budget);
    }

    #[doc = include_str!("diagnostics.doc/frame_budget.md")]
    #[must_use]
    pub fn frame_budget(&self) -> FrameBudget {
        self.telemetry.budget()
    }

    #[doc = include_str!("diagnostics.doc/inspect_mode.md")]
    #[must_use]
    pub const fn inspect_mode(&self) -> bool {
        self.inspector.mode()
    }

    #[doc = include_str!("diagnostics.doc/set_inspect_mode.md")]
    pub fn set_inspect_mode(&mut self, on: bool) {
        if self.inspector.set_mode(on) {
            self.request_frame(FrameReason::Input);
        }
    }

    #[doc = include_str!("diagnostics.doc/inspect_snapshot.md")]
    #[must_use]
    pub fn inspect_snapshot(&self) -> TreeInspection {
        TreeInspector::snapshot(
            &self.tree,
            &self.computed,
            &self.boxes,
            &self.generated_content,
            &self.scroll,
        )
    }

    #[doc = include_str!("diagnostics.doc/hovered_inspect.md")]
    #[must_use]
    pub fn hovered_inspect(&self) -> Option<InspectInfo> {
        TreeInspector::info_at(&self.tree, &self.boxes, self.hovered())
    }
}
