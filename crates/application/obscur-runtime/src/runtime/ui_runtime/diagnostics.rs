#![doc = include_str!("../../../docs/api/runtime/ui_runtime/diagnostics.doc/_module.md")]

use obscur_diagnostics::{Diagnostic, DiagnosticSink, TelemetrySink};
use obscur_geometry::Point;
use obscur_identity::{InternedSymbol, NodeId};
use obscur_interaction::event::{EventOrigin, EventTrust, UiEvent};
use obscur_interaction::spatial::HitChain;

use crate::frame::FrameReason;
use crate::orchestration::{InspectInfo, TreeInspection};
use crate::runtime::{CommandDispatchOutcome, FrameBudget, PointerLandingResolver, TreeInspector};

use super::UiRuntime;

const COMMAND_TRACE_LIMIT: usize = 128;

/// Metadata from a routed command.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CommandTrace {
    /// Event source category.
    pub origin: EventOrigin,
    /// Event trust supplied by the input path.
    pub trust: EventTrust,
    /// Origin node for routing, when present.
    pub target: Option<NodeId>,
    /// Command symbol.
    pub command: InternedSymbol,
    /// Whether any handler ran.
    pub handled: bool,
    /// Whether a handler prevented the default.
    pub default_prevented: bool,
}

impl UiRuntime {
    /// Enables a bounded command history for diagnostics, or clears and disables it.
    pub fn set_command_trace_enabled(&mut self, enabled: bool) {
        if enabled {
            self.command_trace.get_or_insert_with(Default::default);
        } else {
            self.command_trace = None;
        }
    }

    /// Returns the captured command metadata in dispatch order, when enabled.
    pub fn command_trace(&self) -> Option<impl ExactSizeIterator<Item = &CommandTrace>> {
        self.command_trace.as_ref().map(|trace| trace.iter())
    }

    pub(super) fn trace_command(
        &mut self,
        target: Option<NodeId>,
        command: InternedSymbol,
        event: UiEvent,
        outcome: CommandDispatchOutcome,
    ) {
        let Some(trace) = &mut self.command_trace else {
            return;
        };
        if trace.len() == COMMAND_TRACE_LIMIT {
            trace.pop_front();
        }
        trace.push_back(CommandTrace {
            origin: event.origin(),
            trust: event.trust(),
            target,
            command,
            handled: outcome.anything_ran,
            default_prevented: outcome.default_prevented,
        });
    }

    #[doc = include_str!("../../../docs/api/runtime/ui_runtime/diagnostics.doc/set_diagnostic_sink.md")]
    pub fn set_diagnostic_sink(&mut self, sink: Box<dyn DiagnosticSink>) {
        self.diagnostics = sink;
    }

    #[doc = include_str!("../../../docs/api/runtime/ui_runtime/diagnostics.doc/report_diagnostic.md")]
    pub fn report_diagnostic(&self, diagnostic: &Diagnostic) {
        self.diagnostics.emit(diagnostic);
    }

    #[doc = include_str!("../../../docs/api/runtime/ui_runtime/diagnostics.doc/set_telemetry_sink.md")]
    pub fn set_telemetry_sink(&mut self, sink: Box<dyn TelemetrySink>) {
        self.telemetry.set_sink(sink);
    }

    #[doc = include_str!("../../../docs/api/runtime/ui_runtime/diagnostics.doc/set_frame_budget.md")]
    pub fn set_frame_budget(&mut self, budget: FrameBudget) {
        self.telemetry.set_budget(budget);
    }

    /// Returns the per-phase cost budget checked after each frame phase.
    #[must_use]
    pub fn frame_budget(&self) -> FrameBudget {
        self.telemetry.budget()
    }

    /// Returns whether the in-window inspect overlay is active.
    #[must_use]
    pub const fn inspect_mode(&self) -> bool {
        self.inspector.mode()
    }

    #[doc = include_str!("../../../docs/api/runtime/ui_runtime/diagnostics.doc/set_inspect_mode.md")]
    pub fn set_inspect_mode(&mut self, on: bool) {
        if self.inspector.set_mode(on) {
            self.request_frame(FrameReason::Input);
        }
    }

    #[doc = include_str!("../../../docs/api/runtime/ui_runtime/diagnostics.doc/inspect_snapshot.md")]
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

    /// Returns the geometric hit chain for a surface point in the same coordinates
    /// as pointer events. The target comes first, followed by its logical ancestors.
    /// The query leaves capture and hover state untouched. For an ancestor without
    /// a layout box, its point is expressed in folded content coordinates.
    #[must_use]
    pub fn hit_test(&self, position: Point) -> HitChain {
        PointerLandingResolver::new(
            self.surface,
            &self.tree,
            &self.computed,
            &self.boxes,
            &self.scroll,
            &self.hits,
        )
        .hit_chain(position)
    }

    /// Returns a readout of the node currently under the pointer — its identity
    /// (CSS `type#id.class`), component id, and absolute box — for a host to
    /// print alongside the in-window highlight (spec §7.6). `None` when nothing
    /// is hovered or the hovered node has no retained box yet.
    #[must_use]
    pub fn hovered_inspect(&self) -> Option<InspectInfo> {
        TreeInspector::info_at(&self.tree, &self.boxes, self.hovered())
    }
}
