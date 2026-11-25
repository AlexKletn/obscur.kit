//! The frame's structural phase: [`ReconcilePhase`].

use obscur_diagnostics::{
    Diagnostic, DiagnosticCode, DiagnosticSeverity, DiagnosticSink, DiagnosticSubsystem,
    RecoveryAction,
};
use obscur_identity::NodeId;
use obscur_tree::reconciliation::{Reconciler, ReconciliationPlan};
use obscur_tree::tree::RetainedTree;
use obscur_view::view::{View, has_outlet, has_virtual_collection};

use super::reconcile_counts::ReconcileCounts;
use super::{
    ComponentRenderPass, DesiredView, FramePhase, FrameTelemetry, RestylePhase, ScrollOffsets,
    VirtualLists,
};
use crate::component::ComponentStore;
use crate::frame::FrameContext;
use crate::transaction::ReconcileEnactor;

#[doc = include_str!("../../docs/api/runtime/reconcile_phase.doc/ReconcilePhase.md")]
pub(crate) struct ReconcilePhase<'a> {
    /// The retained tree this phase commits into.
    tree: &'a mut RetainedTree,
    /// The reconciler the desired tree is diffed with.
    reconciler: &'a Reconciler,
    /// The transaction boundary the root reconcile is enacted through (spec §3.2),
    /// so the live tree mutation is staged, validated and versioned rather than
    /// applied directly.
    enactor: &'a mut ReconcileEnactor,
    /// The mounted component instances synced against this frame's structural
    /// deltas and then rendered.
    components: &'a mut ComponentStore,
    /// The registered `<VirtualFor>` lists, whose windows are realized against
    /// this frame's viewport before anything reconciles.
    virtual_lists: &'a mut VirtualLists,
    /// The scroll offsets a virtual list whose model was replaced resets, in the
    /// same call that computes the window from them.
    scroll: &'a mut ScrollOffsets,
    /// Where the reconcile phase's span and node count land (spec §7.9).
    telemetry: &'a mut FrameTelemetry,
    /// Where a stray `<slot>` outlet and a duplicated sibling key are reported
    /// (spec §1.6, §7.8).
    diagnostics: &'a dyn DiagnosticSink,
}

impl<'a> ReconcilePhase<'a> {
    /// Bundles the state one reconcile phase reads and writes.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        tree: &'a mut RetainedTree,
        reconciler: &'a Reconciler,
        enactor: &'a mut ReconcileEnactor,
        components: &'a mut ComponentStore,
        virtual_lists: &'a mut VirtualLists,
        scroll: &'a mut ScrollOffsets,
        telemetry: &'a mut FrameTelemetry,
        diagnostics: &'a dyn DiagnosticSink,
    ) -> Self {
        Self {
            tree,
            reconciler,
            enactor,
            components,
            virtual_lists,
            scroll,
            telemetry,
            diagnostics,
        }
    }

    #[doc = include_str!("../../docs/api/runtime/reconcile_phase.doc/run.md")]
    pub(crate) fn run(
        &mut self,
        ctx: &FrameContext,
        desired: DesiredView<'_>,
        viewport_scroll_y: f32,
        previous_sheets: Option<&[obscur_style::matching::Stylesheet]>,
        detailed: bool,
    ) -> (bool, Option<Vec<NodeId>>) {
        // Materialize the desired tree HERE — after phase 3's input dispatch,
        // not before it. A reactive frame's builder reads current state, and a
        // dispatched input handler may have just committed to that state, so
        // building any earlier would reconcile the pre-dispatch view and paint
        // the handler's mutation a frame late (§3.4: dispatch precedes commit).
        let built_view;
        let resolved: &View = match desired {
            DesiredView::Ready(view) => view,
            DesiredView::Build(build) => {
                built_view = build();
                &built_view
            }
        };
        // §4.9: realize every `<VirtualFor>` HERE — the one place that owns the
        // viewport. Each marker (spec §2.11) is replaced by a window of its
        // template whose length comes from `VirtualLayout` against this frame's
        // viewport, so the tree that reconciles below holds tens of rows, not
        // one per item. A view with no marker is returned untouched.
        let realized_view;
        let desired: &View = if has_virtual_collection(resolved.root().nodes()) {
            // The scroll of a list whose content the app replaced is reset in the
            // same call that computes the window from it, so the two cannot come
            // apart — see [`VirtualLists::realize`].
            realized_view = self.virtual_lists.realize(
                resolved,
                ctx.viewport().height,
                viewport_scroll_y,
                self.scroll,
            );
            &realized_view
        } else {
            resolved
        };
        // §2.12: `desired` is the application's own view, so it is nobody's
        // render output and no `project_slots` will ever run over it. An
        // outlet in it — at the root or anywhere in a component host's
        // authored children, which are cloned into a component's output
        // un-projected — is a stray. The reconcile below refuses to
        // materialize it; this is what says so out loud.
        self.report_stray_outlets(desired);
        let (commit, reconcile_plan) = self
            .enactor
            .commit_root(self.reconciler, self.tree, desired)
            .expect("a root reconcile is a single, structurally-valid ReplaceRoot");
        let changed = !commit.is_empty();
        let mut counts = detailed.then(ReconcileCounts::default);
        if let Some(counts) = &mut counts {
            counts.record(&reconcile_plan);
        }
        // The restyle roots this reconcile implies, so the style phase can
        // restyle what the tree edit touched instead of everything. `None`
        // means the edit could not be bounded and the full pass is the only
        // correct answer.
        self.emit_duplicate_key_faults(&reconcile_plan);
        self.virtual_lists.track_containers(self.tree);
        let mut roots = RestylePhase::roots_of(self.tree, &reconcile_plan, previous_sheets);
        // 1b. Mount an instance for every inserted component-backed node and
        //     unmount every removed one, from the plan's structural deltas.
        //     A host that registered no component kinds pays nothing here.
        self.components.sync(self.tree, &reconcile_plan);
        // 1c. Render every mounted component and reconcile its output into
        //     its own children, so a component-backed node's subtree exists
        //     before style/layout read the tree.
        let (rendered, component_roots) = ComponentRenderPass::new(
            self.tree,
            self.components,
            self.reconciler,
            self.enactor,
            self.telemetry,
            self.diagnostics,
        )
        .run(previous_sheets, counts.as_mut());
        // A component sub-reconcile edits the tree through its own plan. That
        // plan used to be invisible from here, so the only correct answer was to
        // widen to the whole tree — and in an application built out of
        // components, a pressed key re-renders one of them and every interactive
        // frame paid a full restyle. The pass hands the roots back now; `None`
        // from either side is still the unbounded case.
        roots = match (roots, component_roots) {
            (Some(mut from_reconcile), Some(from_components)) => {
                from_reconcile.extend(from_components);
                Some(from_reconcile)
            }
            _ => None,
        };
        let changed = changed || rendered;
        if let Some(counts) = counts.filter(|_| changed) {
            println!(
                "[reconcile-work] inserted={} removed={} moved={} updated={} restyle_roots={}",
                counts.inserted,
                counts.removed,
                counts.moved,
                counts.updated,
                roots
                    .as_ref()
                    .map_or_else(|| "full".to_owned(), |roots| roots.len().to_string()),
            );
        }
        // Telemetry: mark the reconcile phase and count the resulting nodes,
        // budgeting the retained-tree size (spec §7.9).
        let nodes = self.tree.len() as u64;
        self.telemetry
            .record_phase(FramePhase::Reconcile, nodes, ctx.id(), self.diagnostics);
        (changed, roots)
    }

    #[doc = include_str!("../../docs/api/runtime/reconcile_phase.doc/report_stray_outlets.md")]
    fn report_stray_outlets(&self, desired: &View) {
        if !has_outlet(desired.root().nodes()) {
            return;
        }
        self.diagnostics.emit(&Diagnostic::new(
            DiagnosticCode::new("view.stray_slot_outlet"),
            DiagnosticSeverity::Error,
            DiagnosticSubsystem::Tree,
            "a `<slot>` outlet was authored outside a component's render output \
             (spec §2.12): an outlet marks where a component's authored children \
             are projected, and only a component's own render is projected, so \
             this one names a slot nothing will ever fill. It is dropped rather \
             than rendered as an empty box. Author the outlet in the `render` of \
             the component that declares the slot",
        ));
    }

    #[doc = include_str!("../../docs/api/runtime/reconcile_phase.doc/emit_duplicate_key_faults.md")]
    fn emit_duplicate_key_faults(&self, plan: &ReconciliationPlan) {
        for duplicate in plan.duplicate_keys() {
            let mut diagnostic = Diagnostic::new(
                DiagnosticCode::new("tree.duplicate_key"),
                DiagnosticSeverity::Warning,
                DiagnosticSubsystem::Tree,
                "several sibling children claim one key, so the group is rebuilt \
                 every frame instead of being reused",
            )
            .with_recovery(RecoveryAction::Fallback);
            diagnostic.context.node = Some(duplicate.parent);
            self.diagnostics.emit(&diagnostic);
        }
    }
}
